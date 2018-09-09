use std::process::exit;
use std::{thread, time};
use std::fs::File;
use std::io::prelude::*;

#[macro_use]
extern crate tera;
use tera::{Context, Tera};

extern crate  rust_docker;
use rust_docker::client::DockerClient;
use rust_docker::api::containers::Containers;
use rust_docker::api::containers::Container;

const TERRA_TEMPLATE : &'static str = "
<ul>
{% for url in urls %}
<li><a href='{{url.0}}'>{{url.1}}</a></li>
{%endfor%}
<ul>
";

fn create_dockerclient() -> DockerClient {
    match DockerClient::new("unix:///var/run/docker.sock") {
        Ok(a) => a,
        Err(err) => {
            println!("err {}", err);
            exit(1);
        }
    }
}

// loop every 10s
//     check for hostnames in docker images
//     construct dictionary
//     construct template with jinja 2
//     put template in /usr/share/nginx/html/
// done
fn list_running_containers(client: &DockerClient) -> Vec<Container> {
    match client.list_running_containers(None) {
        Ok(containers) => containers,
        Err(err) => {println!("An error occured : {}", err); exit(2)}
    }
}


fn filter_images_with_env(image_result: &std::result::Result<rust_docker::api::containers::ContainerDetails, std::string::String>) -> std::option::Option<(std::string::String, std::string::String)> {
    let image = match image_result {
        Ok(image) => image,
        Err(err) => {println!("An error occured : {}", err); exit(4)}
    };
    println!("{:?}", image.Name);
    let environment = &image.Config.Env;
    let mut hostnames = environment.iter().filter_map(|ref e| e.split_terminator("VIRTUAL_HOST=").nth(1));
    while let Some(hostname) = hostnames.next() {
        return Some((hostname.to_string(), image.Name.clone()))
    }
    None
}

fn create_tera_renderer() -> tera::Tera {
    let mut tera = Tera::default();
    tera.add_raw_template("receptionist", TERRA_TEMPLATE).unwrap();
    tera
}

fn write_to_file(input: &str) -> std::io::Result<()> {
    let mut file = File::create("/data/index.html")?;
    file.write_all(input.as_bytes())?;
    Ok(())
}
fn main() {
    println!("starting app");
    let client = create_dockerclient();
    println!("created client");
    let sleep_time = time::Duration::from_secs(10);
    let mut tera = create_tera_renderer();

    loop {
    println!("entered loop");
        let containers = list_running_containers(&client);
        let urls : Vec<_> = containers.iter()
            .map(|ref x| client.inspect_container(&x.Id))
            .filter_map(|ref x| filter_images_with_env(x))
            .collect();


        let mut context = Context::new();
        context.add("urls", &urls);
        if let Ok(webpage) = tera.render("receptionist", &context) {
            write_to_file(&webpage);
        }
        thread::sleep(sleep_time);
    }
}
