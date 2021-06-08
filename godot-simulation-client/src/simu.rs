use crate::godot::*;
use crate::serde::*;
use crate::state::*;
use ompas_lisp::core::LEnv;
use ompas_lisp::structs::LError::{SpecialError, WrongNumberOfArgument, WrongType};
use ompas_lisp::structs::{GetModule, LError, LValue, Module, NameTypeLValue};
use ompas_modules::doc::{Documentation, LHelp};
use ompas_modules::io::TOKIO_CHANNEL_SIZE;
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, Mutex};

/*
LANGUAGE
*/

const MOD_GODOT: &str = "mod-godot";

//commands

const OPEN_COM: &str = "open-com-godot";
const LAUNCH_GODOT: &str = "launch-godot";
const EXEC_GODOT: &str = "exec-godot";

//State variables
//Lambda functions that will be added natively to the environment
//Depends on module state and function get-state.
//Robot

//Lambdas for robots.

//coordinates
const LAMBDA_ROBOT_COORDINATES: &str = "(define robot.coordinates (lambda (x)\
                                                        (get-map (get-state dynamic) ((quote robot.coordinates) x))))";
const SF_ROBOT_COORDINATES: &str = "robot.coordinates";
const DOC_SF_COORDINATES: &str = "Return the coordinates (float float) of a robot";
const DOC_SF_ROBOT_COORDINATES_VERBOSE: &str = "Example: (robot.coordinates robot0)";

//battery
const LAMBDA_ROBOT_BATTERY: &str = "(define robot.battery (lambda (x)\
                                                        (get-map (get-state dynamic) (list (quote robot.battery) x))))";
const SF_ROBOT_BATTERY: &str = "battery";
const DOC_SF_ROBOT_BATTERY: &str = "Return the battery level (float) in [0;1] of a robot.";
const DOC_SF_ROBOT_BATTERY_VERBOSE: &str = "Example: (robot.battery robot0)";

//rotation
const LAMBDA_ROBOT_ROTATION: &str = "(define robot.rotation (lambda (x)\
                                                        (get-map (get-state dynamic) (list (quote robot.rotation) x))))";
const SF_ROBOT_ROTATION: &str = "robot.rotation";
const DOC_SF_ROBOT_ROTATION: &str = "Return the rotation value (float) of a robot."; //TODO: Check the interval of the rotation
const DOC_SF_ROBOT_ROTATION_VERBOSE: &str = "Example: (robot.rotation robot0)";

//velocity
const LAMBDA_ROBOT_VELOCITY: &str = "(define robot.velocity (lambda (x)\
                                                        (get-map (get-state dynamic) (list (quote robot.velocity) x))))";
const SF_ROBOT_VELOCITY: &str = "robot.velocity";
const DOC_SF_ROBOT_VELOCITY: &str =
    "Return the velocity value (float float) in x and y of a robot.";
const DOC_SF_ROBOT_VELOCITY_VERBOSE: &str = "Example: (robot.velocity robot0)";

//rotation speed
const LAMBDA_ROBOT_ROTATION_SPEED: &str = "(define robot.rotation_speed (lambda (x)\
                                                        (get-map (get-state dynamic) (list (quote robot.rotation_speed) x))))";
const SF_ROBOT_ROTATION_SPEED: &str = "robot.rotation_speed";
const DOC_SF_ROBOT_ROTATION_SPEED: &str = "Return the rotation speed value (float) of a robot.";
const DOC_SF_ROBOT_ROTATION_SPEED_VERBOSE: &str = "Example: (robot.rotation_speed robot0)";

//in station
const LAMBDA_ROBOT_IN_STATION: &str = "(define robot.in_station (lambda (x)\
                                                        (get-map (get-state dynamic) (list (quote robot.in_station) x))))";
const SF_ROBOT_IN_STATION: &str = "robot.in_station";
const DOC_SF_ROBOT_IN_STATION: &str = "Return true if a robot is in a station.";
const DOC_SF_ROBOT_IN_STATION_VERBOSE: &str = "Example: (in_station robot0)";

//in interact areas
const LAMBDA_ROBOT_IN_INTERACT_AREAS: &str = "(define robot.in_interact_areas (lambda (x)\
                                                        (get-map (get-state dynamic) (list (quote robot.in_interact_areas) x))))";
const SF_ROBOT_IN_INTERACT_AREAS: &str = "robot.in_interact_areas";
const DOC_SF_ROBOT_IN_INTERACT_AREAS: &str = "Return true if a robot is in an interact zone.";
const DOC_SF_ROBOT_IN_INTERACT_AREAS_VERBOSE: &str = "Example: (robot.in_interact_areas robot0)";

//Lambdas for machines.

//coordinates
const LAMBDA_MACHINE_COORDINATES: &str = "(define machine.coordinates (lambda (x)\
                                                          (get-map (get-state static) (list (quote machine.coordinates) x))))";
const SF_MACHINE_COORDINATES: &str = "machine.coordinates";
const DOC_SF_MACHINE_COORDINATES: &str = "todo!";

//coordinates tile
const LAMBDA_MACHINE_COORDINATES_TILE: &str = "(define machine.coordinates_tile (lambda (x)\
                                                          (get-map (get-state static) (list (quote machine.coordinates_tile) x))))";
const SF_MACHINE_COORDINATES_TILE: &str = "machine.coordinates_tile";
const DOC_SF_MACHINE_COORDINATES_TILE: &str = "todo!";

//input belt
const LAMBDA_MACHINE_INPUT_BELT: &str = "(define machine.input_belt (lambda (x)\
                                                        (get-map (get-state static) (list (quote machine.input_belt) x))))";
const SF_MACHINE_INPUT_BELT: &str = "machine.input_belt";
const DOC_SF_MACHINE_INPUT_BELT: &str = "Return the name (symbol) of the input belt of a machine.";
const DOC_SF_MACHINE_INPUT_BELT_VERBOSE: &str = "Example: (machine.input_belt machine0)";

//output belt
const LAMBDA_MACHINE_OUTPUT_BELT: &str = "(define machine.output_belt (lambda (x)\
                                                        (get-map (get-state static) (list (quote machine.output_belt) x))))";
const SF_MACHINE_OUTPUT_BELT: &str = "machine.output_belt";
const DOC_SF_MACHINE_OUTPUT_BELT: &str =
    "Return the name (symbol) of the output belt of a machine.";
const DOC_SF_MACHINE_OUTPUT_BELT_VERBOSE: &str = "Example: (machine.output_belt machine0)";

//processes
const LAMBDA_MACHINE_PROCESSES: &str = "(define machine.processes_list (lambda (x)\
                                                        (get-map (get-state static) (list (quote machine.processes_list) x))))";
const SF_MACHINE_PROCESSES: &str = "machine.processes_list";
const DOC_SF_MACHINE_PROCESSES: &str = "Return the list of processes (id0 id1 ...) of a machine, or return the list of pair (process_id, duration) for a package.";
const DOC_SF_MACHINE_PROCESSES_VERBOSE: &str = "Example:\n
                                        \t- for a machine: (processes machine0) \n\
                                        \t- for a package: (processes package0)";

//progress rate
const LAMBDA_MACHINE_PROGRESS_RATE: &str =  "(define machine.progress_rate (lambda (x)\
                                                        (get-map (get-state dynamic) (list (quote machine.progress_rate) x))))";
const SF_MACHINE_PROGRESS_RATE: &str = "machine.progress_rate";
const DOC_SF_MACHINE_PROGRESS_RATE: &str = "Return the progress rate (float) in [0;1] of a machine. If no task is in progress, the value is 0";
const DOC_SF_MACHINE_PROGRESS_RATE_VERBOSE: &str = "Example: (machine.progress_rate machine0)";

//Lambdas for packages.

//location
const LAMBDA_PACKAGE_LOCATION: &str = "(define package.location (lambda (x)\
                                                  (get-map (get-state dynamic) (list (quote package.location) x))))";
const SF_PACKAGE_LOCATION: &str = "package.location";
const DOC_SF_PACKAGE_LOCATION: &str = "Return the location (symbol) of a package.";
const DOC_SF_PACKAGE_LOCATION_VERBOSE: &str = "Example: (package.location package0)";

//processes list
const LAMBDA_PACKAGE_PROCESSES_LIST: &str = "(define package.processes_list (lambda (x)\
                                                  (get-map (get-state dynamic) (list (quote package.processes_list) x))))";
const SF_PACKAGE_PROCESSES_LIST: &str = "package.processes_list";
const DOC_SF_PACKAGE_PROCESSES_LIST: &str = "Return the location (symbol) of a package.";
const DOC_SF_PACKAGE_PROCESSES_LIST_VERBOSE: &str = "Example: (package.processes_list package0)";

//Lambdas for belts.

//belt type
const LAMBDA_BELT_BELT_TYPE: &str = "(define belt.belt_type (lambda (x)\
                                                    (get-map (get-state static) (list (quote belt.belt_type) x)))))";
const SF_BELT_BELT_TYPE: &str = "belt_type";
const DOC_SF_BELT_BELT_TYPE: &str = "Return the belt type (symbol) in {input, output} of a belt.";
const DOC_SF_BELT_BELT_TYPE_VERBOSE: &str = "Example: (belt_type belt0)";

//polygons
const LAMBDA_BELT_POLYGONS: &str = "(define belt.polygons (lambda (x)\
                                                    (get-map (get-state static) (list (quote belt.polygons) x)))))";
const SF_BELT_POLYGONS: &str = "belt.polygons";
const DOC_SF_BELT_POLYGONS: &str =
    "Return the coordinates of the polygon [(float float)] that represent the parking area";
const DOC_SF_BELT_POLYGONS_VERBOSE: &str = "Example: (belt.polygons parking_area0)"; //TODO: check the name of the parking are

//cells
const LAMBDA_BELT_CELLS: &str =
    "(define belt.cells (lambda (b) (get-map (get-state static) (list (quote belt.cells) b))))";
const SF_BELT_CELLS: &str = "belt.cells";
const DOC_SF_BELT_CELLS: &str = "todo!";

//interact areas
const LAMBDA_BELT_INTERACT_AREAS: &str = "(define belt.interact_areas (lambda (b) (get-map (get-state static) (list (quote belt.interact_areas) b))))";
const SF_BELT_INTERACT_AREAS: &str = "belt.interact_ares";
const DOC_SF_BELT_INTERACT_AREAS: &str = "todo!";

//packages list
const LAMBDA_BELT_PACKAGES_LIST: &str = "(define belt.packages_list (lambda (x)\
                                                (get-map (get-state dynamic) (list (quote belt.packages_list) x)))))";
const SF_BELT_PACKAGES_LIST: &str = "belt.packages_list";
const DOC_SF_BELT_PACKAGES_LIST: &str = "Return the package list [symbol] on a belt.";
const DOC_SF_BELT_PACKAGES_LIST_VERBOSE: &str = "Example: (belt.packages_list package0)";

//Lambdas for parking areas.

//polygons
const LAMBDA_PARKING_AREA_POLYGONS: &str = "(define parking_area.polygons (lambda (x)\
                                                                         (get-map (get-state static) (list (quote parking_area.polygons) x)))))";
const SF_PARKING_AREA_POLYGONS: &str = "parking_area.polygons";
const DOC_SF_PARKING_AREA_POLYGONS: &str = "todo!";

//cells
const LAMBDA_PARKING_AREA_CELLS: &str = "(define parking_area.cells (lambda (x)\
                                                                         (get-map (get-state static) (list (quote parking_area.cells) x)))))";
const SF_PARKING_AREA_CELLS: &str = "parking_area.cells";
const DOC_SF_PARKING_AREA_CELLS: &str = "todo!";

//Lambdas interact areas.

//polygons
const LAMBDA_INTERACT_AREA_POLYGONS: &str = "(define interact_area.polygons (lambda (x)\
                                                           (get-map (get-state static) (list (quote interact_area.polygons) x)))))";
const SF_INTERACT_AREA_POLYGONS: &str = "interact_area.polygons";
const DOC_SF_INTERACT_AREA_POLYGONS: &str = "todo!";

//cells
const LAMBDA_INTERACT_AREA_CELLS: &str = "(define interact_area.cells (lambda (x)\
                                                           (get-map (get-state static) (list (quote interact_area.cells) x)))))";
const SF_INTERACT_AREA_CELLS: &str = "interact_area.cells";
const DOC_SF_INTERACT_AREA_CELLS: &str = "todo!";

//belt
const LAMBDA_INTERACT_AREA_BELT: &str = "(define interact_area.belt (lambda (x)\
                                                           (get-map (get-state static) (list (quote interact_area.belt) x)))))";
const SF_INTERACT_AREA_BELT: &str = "interact_area.belt";
const DOC_SF_INTERACT_AREA_BELT: &str = "todo!";

//Lambdas for actions.

//rotation
const LAMBDA_DO_ROTATION: &str =
    "(define robot.do_rotation (lambda (r a w) (exec-godot do_rotation r a w)))";
const ACTION_DO_ROTATION: &str = "robot.do_rotation";
const DOC_ACTION_DO_ROTATION: &str = "todo!";

//pick
const LAMBDA_PICK: &str = "(define robot.pick (lambda (r a w) (exec-godot pick r a w)))";
const ACTION_PICK: &str = "robot.pick";
const DOC_ACTION_PICK: &str = "todo!";

//place
const LAMBDA_PLACE: &str = "(define robot.place (lambda (r a w) (exec-godot place r a w)))";
const ACTION_PLACE: &str = "robot.place";
const DOC_ACTION_PLACE: &str = "todo!";

//navigate_to
const LAMBDA_NAVIGATE_TO: &str =
    "(define robot.navigate_to (lambda (r a w) (exec-godot navigate_to r a w)))";
const ACTION_NAVIGATE_TO: &str = "robot.navigate_to";
const DOC_ACTION_NAVIGATE_TO: &str = "todo!";

//Constants

//Documentation
const DOC_MOD_GODOT: &str = "Module to use the simulator developped in godot. It add functions and lambda for the state variables";
const DOC_MOD_GODOT_VERBOSE: &str = "functions:\n\
                                     \t- open-com-godot\n\
                                     \t- launch-godot (not implemented yet)\n\
                                     \t- exec-godot\n\
                                     \t- get-state\n\n\
                                     lambdas for the state functions:\n\
                                     \t- for a robot: coordinates, battery, rotation, velocity, rotation_speed, in_station, in_interact\n\
                                     \t- for a machine: coordinates, input_belt, output_belt, processes, progress_rate\n\
                                     \t- for a package: location, processes\n\
                                     \t- for a belt: coordinates, belt_type, polygons, packages_list\n\
                                     \t- for a parking are: polygon";

const DOC_OPEN_COM: &str = "Connect via TCP to the simulator.";
const DOC_OPEN_COM_VERBOSE: &str = "Default socket address is 127.0.0.1:10000, but you can provide the IP address and the port that you want.";
const DOC_LAUNCH_GODOT: &str = "Not yet implemented";
const DOC_EXEC_GODOT: &str = "Send a command to the simulator";
const DOC_EXEC_GODOT_VERBOSE: &str = "Available commands:\n\
                                       \t- Navigate to : [navigate_to', robot_name, destination_x, destination_y] \n\
                                       \t- Pick : ['pickup', robot_name] \n\
                                       \t- Place : ['place', robot_name] \n\
                                       \t- Rotation : ['do_rotation', robot_name, angle, speed] \n\n\
                                       Example: (exec-godot navigate_to robot0 2 3)";

//functions
//const SET_STATE: &str = "set-state";
const GET_STATE: &str = "get-state";
//const UPDATE_STATE: &str = "update-state";

//Documentation

/*const DOC_SET_STATE: &str = "Set a map as a the new state";
const DOC_SET_VERBOSE: &str = "Takes 2 arguments:\n\
                                \t- the type of state to set: {static, dynamic}\n\
                                \t- the map that will overwrite the current state";*/
const DOC_GET_STATE: &str = "Return the current state.";
const DOC_GET_STATE_VERBOSE: &str = "Takes an optional argument: {static, dynamic}";
/*const DOC_UPDATE_STATE: &str = "Update the current state with facts of a map";
const DOC_UPDATE_VERBOSE: &str = "Takes 2 arguments:\n\
                                \t- the type of state to set: {static, dynamic}\n\
                                \t- the map that will overwrite the current state";*/

#[derive(Default)]
pub struct SocketInfo {
    pub addr: String,
    pub port: usize,
}

#[derive(Default)]
pub struct CtxGodot {
    socket_info: SocketInfo,
    sender_li: Option<Sender<String>>,
    sender_socket: Option<Sender<String>>,
    state: Arc<Mutex<GodotState>>,
}

impl CtxGodot {
    pub fn set_sender_li(&mut self, sender: Sender<String>) {
        self.sender_li = Some(sender);
    }

    pub fn set_socket_info(&mut self, socket_info: SocketInfo) {
        self.socket_info = socket_info;
    }

    pub fn get_sender_li(&self) -> &Option<Sender<String>> {
        &self.sender_li
    }

    pub fn get_socket_info(&self) -> &SocketInfo {
        &self.socket_info
    }

    pub fn get_sender_socket(&self) -> &Option<Sender<String>> {
        &self.sender_socket
    }

    pub async fn get_state(&self, _type: Option<StateType>) -> LState {
        self.state.lock().await.get_state(_type)
    }

    pub fn get_ref_state(&self) -> Arc<Mutex<GodotState>> {
        self.state.clone()
    }
}

impl GetModule for CtxGodot {
    fn get_module(self) -> Module {
        let raw_lisp = vec![
            LAMBDA_ROBOT_BATTERY,
            LAMBDA_ROBOT_COORDINATES,
            LAMBDA_ROBOT_IN_INTERACT_AREAS,
            LAMBDA_ROBOT_IN_STATION,
            LAMBDA_ROBOT_ROTATION,
            LAMBDA_ROBOT_ROTATION_SPEED,
            LAMBDA_ROBOT_VELOCITY,
            LAMBDA_BELT_BELT_TYPE,
            LAMBDA_BELT_PACKAGES_LIST,
            LAMBDA_BELT_POLYGONS,
            LAMBDA_BELT_INTERACT_AREAS,
            LAMBDA_BELT_CELLS,
            LAMBDA_PACKAGE_LOCATION,
            LAMBDA_MACHINE_PROGRESS_RATE,
            LAMBDA_MACHINE_PROCESSES,
            LAMBDA_MACHINE_OUTPUT_BELT,
            LAMBDA_MACHINE_INPUT_BELT,
            LAMBDA_MACHINE_COORDINATES,
            LAMBDA_MACHINE_COORDINATES_TILE,
            LAMBDA_PARKING_AREA_POLYGONS,
            LAMBDA_PARKING_AREA_CELLS,
            LAMBDA_PACKAGE_PROCESSES_LIST,
            LAMBDA_INTERACT_AREA_BELT,
            LAMBDA_INTERACT_AREA_CELLS,
            LAMBDA_INTERACT_AREA_POLYGONS,
            LAMBDA_DO_ROTATION,
            LAMBDA_NAVIGATE_TO,
            LAMBDA_PICK,
            LAMBDA_PLACE,
        ]
        .into();

        let mut module = Module {
            ctx: Box::new(self),
            prelude: vec![],
            raw_lisp,
            label: MOD_GODOT,
        };

        module.add_mut_fn_prelude(OPEN_COM, Box::new(open_com));
        module.add_fn_prelude(LAUNCH_GODOT, Box::new(launch_godot));
        module.add_fn_prelude(EXEC_GODOT, Box::new(exec_godot));
        module.add_fn_prelude(GET_STATE, Box::new(get_state));
        //module.add_mut_fn_prelude(SET_STATE, Box::new(set_state));
        //module.add_mut_fn_prelude(UPDATE_STATE, Box::new(update_state));

        module
    }
}

impl Documentation for CtxGodot {
    fn documentation() -> Vec<LHelp> {
        vec![
            LHelp::new(MOD_GODOT, DOC_MOD_GODOT, Some(DOC_MOD_GODOT_VERBOSE)),
            LHelp::new(OPEN_COM, DOC_OPEN_COM, Some(DOC_OPEN_COM_VERBOSE)),
            LHelp::new(LAUNCH_GODOT, DOC_LAUNCH_GODOT, None),
            LHelp::new(EXEC_GODOT, DOC_EXEC_GODOT, Some(DOC_EXEC_GODOT_VERBOSE)),
            //Add doc for state functions
            LHelp::new(
                SF_ROBOT_BATTERY,
                DOC_SF_ROBOT_BATTERY,
                Some(DOC_SF_ROBOT_BATTERY_VERBOSE),
            ),
            LHelp::new(
                SF_ROBOT_COORDINATES,
                DOC_SF_COORDINATES,
                Some(DOC_SF_ROBOT_COORDINATES_VERBOSE),
            ),
            LHelp::new(
                SF_ROBOT_IN_INTERACT_AREAS,
                DOC_SF_ROBOT_IN_INTERACT_AREAS,
                Some(DOC_SF_ROBOT_IN_INTERACT_AREAS_VERBOSE),
            ),
            LHelp::new(
                SF_ROBOT_IN_STATION,
                DOC_SF_ROBOT_IN_STATION,
                Some(DOC_SF_ROBOT_IN_STATION_VERBOSE),
            ),
            LHelp::new(
                SF_ROBOT_ROTATION,
                DOC_SF_ROBOT_ROTATION,
                Some(DOC_SF_ROBOT_ROTATION_VERBOSE),
            ),
            LHelp::new(
                SF_ROBOT_ROTATION_SPEED,
                DOC_SF_ROBOT_ROTATION_SPEED,
                Some(DOC_SF_ROBOT_ROTATION_SPEED_VERBOSE),
            ),
            LHelp::new(
                SF_ROBOT_VELOCITY,
                DOC_SF_ROBOT_VELOCITY,
                Some(DOC_SF_ROBOT_VELOCITY_VERBOSE),
            ),
            LHelp::new(SF_MACHINE_COORDINATES, DOC_SF_MACHINE_COORDINATES, None),
            LHelp::new(
                SF_MACHINE_COORDINATES_TILE,
                DOC_SF_MACHINE_COORDINATES_TILE,
                None,
            ),
            LHelp::new(
                SF_MACHINE_INPUT_BELT,
                DOC_SF_MACHINE_INPUT_BELT,
                Some(DOC_SF_MACHINE_INPUT_BELT_VERBOSE),
            ),
            LHelp::new(
                SF_PACKAGE_LOCATION,
                DOC_SF_PACKAGE_LOCATION,
                Some(DOC_SF_PACKAGE_LOCATION_VERBOSE),
            ),
            LHelp::new(
                SF_BELT_PACKAGES_LIST,
                DOC_SF_BELT_PACKAGES_LIST,
                Some(DOC_SF_BELT_PACKAGES_LIST_VERBOSE),
            ),
            LHelp::new(
                SF_PACKAGE_PROCESSES_LIST,
                DOC_SF_PACKAGE_PROCESSES_LIST,
                Some(DOC_SF_PACKAGE_PROCESSES_LIST_VERBOSE),
            ),
            LHelp::new(
                SF_MACHINE_OUTPUT_BELT,
                DOC_SF_MACHINE_OUTPUT_BELT,
                Some(DOC_SF_MACHINE_OUTPUT_BELT_VERBOSE),
            ),
            LHelp::new(
                SF_MACHINE_PROCESSES,
                DOC_SF_MACHINE_PROCESSES,
                Some(DOC_SF_MACHINE_PROCESSES_VERBOSE),
            ),
            LHelp::new(
                SF_MACHINE_PROGRESS_RATE,
                DOC_SF_MACHINE_PROGRESS_RATE,
                Some(DOC_SF_MACHINE_PROGRESS_RATE_VERBOSE),
            ),
            LHelp::new(
                SF_BELT_POLYGONS,
                DOC_SF_BELT_POLYGONS,
                Some(DOC_SF_BELT_POLYGONS_VERBOSE),
            ),
            LHelp::new(
                SF_BELT_BELT_TYPE,
                DOC_SF_BELT_BELT_TYPE,
                Some(DOC_SF_BELT_BELT_TYPE_VERBOSE),
            ),
            LHelp::new(SF_BELT_CELLS, DOC_SF_BELT_CELLS, None),
            LHelp::new(SF_BELT_INTERACT_AREAS, DOC_SF_BELT_INTERACT_AREAS, None),
            LHelp::new(SF_INTERACT_AREA_BELT, DOC_SF_INTERACT_AREA_BELT, None),
            LHelp::new(SF_INTERACT_AREA_CELLS, DOC_SF_INTERACT_AREA_CELLS, None),
            LHelp::new(
                SF_INTERACT_AREA_POLYGONS,
                DOC_SF_INTERACT_AREA_POLYGONS,
                None,
            ),
            LHelp::new(SF_PARKING_AREA_POLYGONS, DOC_SF_PARKING_AREA_POLYGONS, None),
            LHelp::new(SF_PARKING_AREA_CELLS, DOC_SF_PARKING_AREA_CELLS, None),
            LHelp::new(GET_STATE, DOC_GET_STATE, Some(DOC_GET_STATE_VERBOSE)),
            LHelp::new(ACTION_DO_ROTATION, DOC_ACTION_DO_ROTATION, None),
            LHelp::new(ACTION_NAVIGATE_TO, DOC_ACTION_NAVIGATE_TO, None),
            LHelp::new(ACTION_PICK, DOC_ACTION_PICK, None),
            LHelp::new(ACTION_PLACE, DOC_ACTION_PLACE, None),
        ]
    }
}

/*
Functions
 */

fn open_com(args: &[LValue], _: &mut LEnv, ctx: &mut CtxGodot) -> Result<LValue, LError> {
    let socket_addr: SocketAddr = match args.len() {
        0 => "127.0.0.1:10000".parse().unwrap(),
        2 => {
            let addr = match &args[0] {
                LValue::Symbol(s) => s.clone(),
                lv => return Err(WrongType(lv.clone(), lv.into(), NameTypeLValue::Symbol)),
            };

            let port: usize = match &args[1] {
                LValue::Number(n) => n.into(),
                lv => return Err(WrongType(lv.clone(), lv.into(), NameTypeLValue::Usize)),
            };

            format!("{}:{}", addr, port).parse().unwrap()
        }
        _ => return Err(WrongNumberOfArgument(args.into(), args.len(), 2..2)),
    };

    let (tx, rx) = mpsc::channel(TOKIO_CHANNEL_SIZE);
    ctx.sender_socket = Some(tx);

    let state = ctx.state.clone();
    tokio::spawn(async move { task_tcp_connection(&socket_addr, rx, state).await });

    Ok(LValue::Nil)
}

fn launch_godot(_: &[LValue], _: &LEnv, ctx: &CtxGodot) -> Result<LValue, LError> {
    let sender = match ctx.get_sender_li() {
        None => return Err(SpecialError("ctx godot has no sender to l.i.".to_string())),
        Some(s) => s.clone(),
    };
    tokio::spawn(async move {
        sender
            .send("(print (quote (launch-godot not implemented yet)))".to_string())
            .await
            .expect("couldn't send via channel");
    });
    Ok(LValue::Nil)
}
/// Commands available
///- Navigate to : ['navigate_to', robot_name, destination_x, destination_y]
///- Pick : ['pickup', robot_name]
///- Place : ['place', robot_name]
///- Rotation : ['do_rotation', robot_name, angle, speed]

fn exec_godot(args: &[LValue], _: &LEnv, ctx: &CtxGodot) -> Result<LValue, LError> {
    let gs = GodotMessageSerde {
        _type: GodotMessageType::RobotCommand,
        data: LValue::List(args.to_vec()).into(),
    };

    let command = serde_json::to_string(&gs).unwrap();

    let sender = match ctx.get_sender_socket() {
        None => {
            return Err(SpecialError(
                "ctx godot has no sender to simulation, try first to (open-com-godot)".to_string(),
            ))
        }
        Some(s) => s.clone(),
    };
    tokio::spawn(async move {
        sender
            .send(command)
            .await
            .expect("couldn't send via channel");
    });
    Ok(LValue::Nil)
}

/*pub fn set_state(args: &[LValue], _: &mut LEnv, ctx: &mut CtxGodot) -> Result<LValue, LError> {
    if args.len() != 2 {
        return Err(WrongNumberOfArgument(args.into(), args.len(), 2..2));
    }

    let first_arg = &args[0];
    let state_type = match String::try_from(first_arg) {
        Ok(s) => match s.as_str() {
            KEY_DYNAMIC => StateType::Dynamic,
            KEY_STATIC => StateType::Static,
            _ => {
                return Err(SpecialError(format!(
                    "Expected keywords {} or {}",
                    KEY_STATIC, KEY_DYNAMIC
                )))
            }
        },
        Err(_) => {
            return Err(WrongType(
                first_arg.clone(),
                first_arg.into(),
                NameTypeLValue::String,
            ))
        }
    };

    match &args[1] {
        LValue::Map(m) => {
            ctx.set_state(m.into(), &state_type);
            Ok(LValue::Nil)
        }
        lv => Err(WrongType(lv.clone(), lv.into(), NameTypeLValue::Map)),
    }
}*/

pub fn get_state(args: &[LValue], _: &LEnv, ctx: &CtxGodot) -> Result<LValue, LError> {
    let _type = match args.len() {
        0 => None,
        1 => match &args[0] {
            LValue::Symbol(s) => match s.as_str() {
                KEY_STATIC => Some(StateType::Static),
                KEY_DYNAMIC => Some(StateType::Dynamic),
                _ => {
                    return Err(SpecialError(format!(
                        "Expected keywords {} or {}",
                        KEY_STATIC, KEY_DYNAMIC
                    )))
                }
            },
            lv => return Err(WrongType(lv.clone(), lv.into(), NameTypeLValue::Symbol)),
        },
        _ => return Err(WrongNumberOfArgument(args.into(), args.len(), 0..1)),
    };

    let handle = tokio::runtime::Handle::current();
    let state = ctx.get_ref_state();
    let result =
        thread::spawn(move || handle.block_on(async move { state.lock().await.get_state(_type) }))
            .join()
            .unwrap();

    Ok(result.into_map())
}

/*///Update the last state with the new facts of the map.
pub fn update_state(args: &[LValue], _: &mut LEnv, ctx: &mut CtxGodot) -> Result<LValue, LError> {
    if args.len() != 2 {
        return Err(WrongNumberOfArgument(args.into(), args.len(), 2..2));
    }

    let first_arg = &args[0];
    let state_type = match String::try_from(first_arg) {
        Ok(s) => match s.as_str() {
            KEY_DYNAMIC => StateType::Dynamic,
            KEY_STATIC => StateType::Static,
            _ => {
                return Err(SpecialError(format!(
                    "Expected keywords {} or {}",
                    KEY_STATIC, KEY_DYNAMIC
                )))
            }
        },
        Err(_) => {
            return Err(WrongType(
                first_arg.clone(),
                first_arg.into(),
                NameTypeLValue::String,
            ))
        }
    };

    match &args[1] {
        LValue::Map(m) => {
            ctx.update_state(m.into(), &state_type);
            Ok(LValue::Nil)
        }
        lv => Err(WrongType(lv.clone(), lv.into(), NameTypeLValue::Map)),
    }
}*/