use std::{thread, time};
use std::fs::File;
use std::io::prelude::*;

extern crate tera;
use tera::{Context, Tera};

extern crate shiplift;
use shiplift::{Containers, Docker};

type ContainerDescription = (std::string::String, std::string::String, std::string::String);
const TERRA_TEMPLATE : &'static str = r#"
<!doctype html>
<html lang="en">
    <head>
        <!-- Required meta tags -->
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1, shrink-to-fit=no">

        <!-- Bootstrap CSS -->
        <link rel="stylesheet" href="/bootstrap.min.css" integrity="sha384-MCw98/SFnGE8fJT3GXwEOngsV7Zt27NXFoaoApmYm81iuXoPkFOJwJ8ERdknLPMO">

        <title>Index - vince.lol</title>
    </head>
    <body>
        <div class="container">
            <h1 class='text-center'>MCP index</h1>
            <div class="row">
            {% for url in urls %}
                <div class="col-sm-12 col-md-6 col-lg-4">
                <a class='nav-link' href='https://{{url.1}}'>
                    <div class="card">
                        <div class="card-body">
                            <h5 class="card-title">{{url.0 | title }}</h5>
                            <p class="card-text text-dark">{% if url.2 is defined %}{{url.2}}{% else %} no description {% endif %}</p>
                        </div>
                    </div>
                </a>
                </div>
            {%endfor%}
            </div>
        </div>
        <script src="/jquery-3.3.1.slim.min.js" integrity="sha384-q8i/X+965DzO0rT7abK41JStQIAqVgRVzpbzo5smXKp4YfRvH+8abtTE1Pi6jizo" defer></script>
        <script src="/popper.min.js" integrity="sha384-ZMP7rVo3mIykV+2+9J3UJ46jBk0WLaUAdn689aCwoqbBJiSnjAK/l8WvCWPIPm49" defer></script>
        <script src="/bootstrap.min.js" integrity="sha384-ChfqqxuZUCnJSK3+MXmPNIyE6ZbWh2IMqE241rYiqJxyMiZ6OW/JmZQ5stwEULTy" defer></script>
    </body>
</html>
"#;


fn create_tera_renderer() -> tera::Tera {
    let mut tera = Tera::default();
    tera.add_raw_template("receptionist", TERRA_TEMPLATE).unwrap();
    tera
}

fn write_to_file(input: &str) -> std::io::Result<()> {
    let mut file = File::create("/usr/share/nginx/html/index.html")?;
    file.write_all(input.as_bytes())?;
    Ok(())
}


fn filter_by_env(containers: &Containers, rep: &shiplift::rep::Container) -> Option<ContainerDescription> {

    if rep.Names.join(" ").contains(&"receptionist".to_string()) {
        return None
    }

    let container = containers.get(&rep.Id);

    if let Ok(details) = container.inspect() {
        let env = details.Config.env();
        if let Some(_) = env.get("NOPUBLISH") {
            return None
        }
        if let Some(domain) = env.get("VIRTUAL_HOST") {
            let description = match env.get("DESCRIPTION") {
                Some(description) => description,
                None => "no description provided"
            };
            return Some((rep.Names[0].trim_matches('/').to_string(), domain.to_string(), description.to_string()))
        }
    }
    None
}

fn main() -> ! {
    println!("starting app");
    let docker = Docker::new();
    println!("created client");
    let sleep_time = time::Duration::from_secs(10);
    let tera = create_tera_renderer();

    println!("starting loop");
    // loop every 10s
    loop {
        let containers = docker.containers();

        // check for hostnames in docker images
        let mut urls : Vec<ContainerDescription> = containers.list(&Default::default()).unwrap().iter()
        // construct dictionary
        .filter_map(|x| filter_by_env(&containers, &x)).collect();
        urls.sort_unstable();
        let urls = urls;

        let mut context = Context::new();
        context.add("urls", &urls);
        // construct template with jinja 2
        if let Ok(webpage) = tera.render("receptionist", &context) {
            //put template in /usr/share/nginx/html/
            if let Err(err) = write_to_file(&webpage) {
                panic!(err);
            }
        }
        thread::sleep(sleep_time);
    }
}
