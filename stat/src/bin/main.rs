use ompas_stat::stat::formatter::SystemStatFormatter;
use ompas_stat::stat::system::SystemRunData;
use ompas_stat::statos_config::StatosConfig;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "Statos", about = "Generation of problems for gripper domain")]
struct Opt {
    #[structopt(short = "c", long = "config")]
    config: PathBuf,
}

pub fn main() {
    println!("Hello, world!");

    let opt = Opt::from_args();

    let str = fs::read_to_string(opt.config).expect("Could not read config file");

    let config: StatosConfig =
        serde_yaml::from_str(&str).expect("Could not deserialize content of config");

    println!("config: {:?}", config);

    for config in config.configs {
        let system_run = SystemRunData::new(&config.input_dirs, config.clone());

        let time = SystemTime::now();
        let stat = system_run.compute_stat();
        for output in &config.outputs {
            let formatter = SystemStatFormatter::new(&stat, &output.configs, &output.fields);

            println!(
                "time to compute stat : {} s",
                time.elapsed().unwrap().as_secs_f32()
            );

            // println!("{}", formatter);

            if let Some(csv_output) = &output.csv_output {
                let csv = formatter.to_csv();

                let mut file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&csv_output)
                    .unwrap();
                file.write_all(csv.as_bytes()).unwrap();
            }

            if let Some(latex_output) = &output.latex_output {
                let csv = formatter.to_latex();

                let mut file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&latex_output)
                    .unwrap();
                file.write_all(csv.as_bytes()).unwrap();
            }
        }
    }
}
