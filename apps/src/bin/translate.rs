use std::path::PathBuf;

use ompas_lisp::core::activate_debug;
use structopt::StructOpt;

//use ompas_modules::robot::CtxRobot;
use ompas_lisp::lisp_interpreter::{LispInterpreter, LispInterpreterConfig};
use ompas_lisp::modules::_type::CtxType;
use ompas_lisp::modules::advanced_math::CtxMath;
use ompas_lisp::modules::doc::{CtxDoc, Documentation};
use ompas_lisp::modules::error::CtxError;
use ompas_lisp::modules::io::CtxIo;
use ompas_lisp::modules::string::CtxString;
use ompas_lisp::modules::utils::CtxUtils;
use ompas_planning::mod_domain::CtxDomain;

pub const TOKIO_CHANNEL_SIZE: usize = 65_384;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Translate",
    about = "A binary to translate RAE Domain into Chronicles"
)]
struct Opt {
    #[structopt(short = "d", long = "debug")]
    debug: bool,

    #[structopt(short = "p", long = "log-path")]
    log: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    println!("Scheme console v0.1");

    let opt: Opt = Opt::from_args();
    println!("{:?}", opt);
    if opt.debug {
        activate_debug();
    }

    //test_lib_model(&opt);
    lisp_interpreter(opt.log).await;
}

pub async fn lisp_interpreter(log: Option<PathBuf>) {
    let mut li = LispInterpreter::new().await;

    let mut ctx_doc = CtxDoc::default();
    let mut ctx_io = CtxIo::default();
    let ctx_math = CtxMath::default();
    let ctx_type = CtxType::default();
    let ctx_utils = CtxUtils::default();
    let ctx_string = CtxString::default();
    let ctx_domain = CtxDomain::new().await;

    //Insert the doc for the different contexts.
    ctx_doc.insert_doc(CtxIo::documentation());
    ctx_doc.insert_doc(CtxMath::documentation());
    ctx_doc.insert_doc(CtxType::documentation());
    ctx_doc.insert_doc(CtxUtils::documentation());
    ctx_doc.insert_doc(CtxString::documentation());
    ctx_doc.insert_doc(CtxDomain::documentation());

    //Add the sender of the channel.
    ctx_io.add_communication(li.subscribe());
    if let Some(pb) = &log {
        ctx_io.set_log_output(pb.clone().into());
    }

    li.import_namespace(CtxError::default())
        .await
        .expect("error loading error");
    li.import_namespace(ctx_utils)
        .await
        .expect("error loading utils");
    li.import_namespace(ctx_doc)
        .await
        .expect("error loading doc");
    li.import_namespace(ctx_io).await.expect("error loading io");
    li.import_namespace(ctx_math)
        .await
        .expect("error loading math");
    li.import_namespace(ctx_type)
        .await
        .expect("error loading type");

    li.import(ctx_string)
        .await
        .expect("error loading ctx string");

    li.import_namespace(ctx_domain)
        .await
        .expect("error loading rae domain");

    li.set_config(LispInterpreterConfig::new(true));

    li.run(log).await;
}
