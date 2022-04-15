use sompas_core::activate_debug;
use sompas_language::NIL;
use sompas_modules::advanced_math::CtxMath;
use sompas_modules::io::CtxIo;
use sompas_modules::string::CtxString;
use sompas_modules::utils::CtxUtils;
use sompas_repl::lisp_interpreter::{LispInterpreter, LispInterpreterConfig};
use std::fs;
use std::path::PathBuf;

use ompas_rae_scheme::rae_user::CtxRae;
use structopt::StructOpt;

pub const TOKIO_CHANNEL_SIZE: usize = 65_384;

#[derive(Debug, StructOpt)]
#[structopt(name = "OMPAS", about = "An acting engine based on RAE.")]
struct Opt {
    #[structopt(short = "d", long = "debug")]
    debug: bool,
    #[structopt(short = "p", long = "log-path")]
    log: Option<PathBuf>,

    #[structopt(short = "s", long = "sim-domain")]
    sim_domain: PathBuf,

    #[structopt(short = "r", long = "rae-log")]
    rae_log: bool,
}

#[tokio::main]
async fn main() {
    println!("OMPAS v0.1");

    let opt: Opt = Opt::from_args();
    println!("{:?}", opt);
    if opt.debug {
        activate_debug();
    }
    //test_lib_model(&opt);
    lisp_interpreter(opt.log, opt.sim_domain, opt.rae_log).await;
}

pub async fn lisp_interpreter(log: Option<PathBuf>, sim_domain: PathBuf, rae_log: bool) {
    let mut li = LispInterpreter::new().await;

    let mut ctx_io = CtxIo::default();
    let ctx_math = CtxMath::default();
    let ctx_utils = CtxUtils::default();
    let ctx_string = CtxString::default();

    //Insert the doc for the different contexts.

    //Add the sender of the channel.
    if let Some(pb) = &log {
        ctx_io.set_log_output(pb.clone().into());
    }

    li.import_namespace(ctx_utils);
    li.import_namespace(ctx_io);
    li.import_namespace(ctx_math);
    li.import(ctx_string);

    let mut com = li.subscribe();
    let str = fs::read_to_string(sim_domain).expect("Something went wrong reading the file");
    //println!("string in file: {}", str);
    com.send(str).await.expect("could not send to LI");
    tokio::spawn(async move {
        let str: String = com
            .recv()
            .await
            .expect("error receiving result of initialisation of domain");
        if str != NIL {
            println!("init com: {}", str)
        }
    });

    let ctx_rae = CtxRae::init_ctx_rae(None, log.clone(), rae_log).await;
    li.import_namespace(ctx_rae);

    li.set_config(LispInterpreterConfig::new(true));

    li.run(log).await;
}
