use std::fs;
//use ompas_gobotsim::mod_godot::CtxGodot;
use ompas_gobotsim::platform::PlatformGobotSim;
use ompas_middleware::logger::{FileDescriptor, LogClient, Logger};
use ompas_middleware::{LogLevel, Master};
use ompas_rae_core::monitor::ModRaeUser;
use ompas_rae_interface::platform::Domain;
use ompas_rae_interface::{LOG_TOPIC_PLATFORM, PLATFORM_CLIENT};
use sompas_core::activate_debug;
use sompas_modules::advanced_math::ModMath;
use sompas_modules::io::ModIO;
use sompas_modules::string::ModString;
use sompas_modules::utils::ModUtils;
use sompas_repl::lisp_interpreter::{LispInterpreter, LispInterpreterConfig};
use std::path::PathBuf;
use structopt::StructOpt;

pub const TOKIO_CHANNEL_SIZE: usize = 100;
pub const LOG_LEVEL: LogLevel = LogLevel::Trace;

#[derive(Debug, StructOpt)]
#[structopt(name = "OMPAS", about = "An acting engine based on RAE.")]
struct Opt {
    #[structopt(short = "d", long = "debug")]
    debug: bool,
    #[structopt(short = "p", long = "log-path")]
    log: Option<PathBuf>,

    #[structopt(short = "G", long = "godot")]
    godot: bool,

    #[structopt(short = "r", long = "rae-log")]
    rae_log: bool,
}

#[tokio::main]
async fn main() {
    println!("OMPAS v0.1");

    let opt: Opt = Opt::from_args();
    println!("{:?}", opt);
    Master::set_log_level(LOG_LEVEL).await;

    if opt.debug {
        Master::set_log_level(LogLevel::Trace).await;
    }

    //test_lib_model(&opt);
    lisp_interpreter(opt.log, opt.godot, opt.rae_log).await;
}

pub async fn lisp_interpreter(log: Option<PathBuf>, godot: bool, rae_log: bool) {
    let mut li = LispInterpreter::new().await;

    let mut ctx_io = ModIO::default();
    let ctx_math = ModMath::default();
    let ctx_utils = ModUtils::default();
    let ctx_string = ModString::default();

    //Insert the doc for the different contexts.

    //Add the sender of the channel.
    if let Some(pb) = &log {
        ctx_io.set_log_output(pb.clone().into());
    }

    li.import_namespace(ctx_utils);
    li.import_namespace(ctx_io);
    li.import_namespace(ctx_math);
    li.import(ctx_string);

    if godot {
        //li.import_namespace(CtxGodot::default());
    } else {
        let ctx_rae = ModRaeUser::new(
            PlatformGobotSim::new(
                Domain::File(
                    "/home/jeremy/CLionProjects/ompas/gobotsim/godot_domain/domain.lisp".into(),
                ),
                false,
                LogClient::new(PLATFORM_CLIENT, LOG_TOPIC_PLATFORM).await,
            ),
            log.clone(),
            rae_log,
        )
        .await;
        li.import_namespace(ctx_rae);
    }

    li.set_config(LispInterpreterConfig::new(true));

    li.run(log.map(|p| FileDescriptor::AbsolutePath(fs::canonicalize(p).unwrap())))
        .await;
    Master::end().await;
}
