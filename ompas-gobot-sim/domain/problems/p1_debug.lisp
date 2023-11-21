(begin
    (def-objects
        (robot0 robot1 robot)
        (machine0 machine1 machine2 machine3 machine4 machine5 input_machine0 output_machine0 machine)
        ;(package0 package1 package)
        (package0 package)
        )

    (def-resources
        robot0 robot1
        machine0 machine1 machine2 machine3 machine4 machine5 input_machine0 output_machine0 machine
        )

    (def-facts
        ((robot.rotation_speed robot1) 0)
        ((machine.progress_rate machine3) 0)
        ((machine.progress_rate machine0) 0)
        ((robot.rotation_speed robot0) 0)
        ((machine.progress_rate machine4) 0)
        ((robot.in_station robot0) true)
        ((robot.velocity robot0) (0 0))
        ((machine.progress_rate input_machine0) 0)
        ((robot.battery robot0) 1)
        ((robot.instance robot1) robot)
        ((machine.progress_rate machine2) 0)
        ((robot.battery robot1) 1)
        ((robot.instance robot0) robot)
        ((instance robot) (robot1 robot0))
        ;((package.instance package1) package)
        ((machine.progress_rate machine5) 0)
        ((robot.in_station robot1) true)
        ((machine.progress_rate output_machine0) 1)
        ((robot.coordinates_tile robot0) (7 16))
        ((robot.velocity robot1) (0 0))
        ((robot.coordinates_tile robot1) (6 16))
        ((robot.rotation robot0) 0)
        ((robot.coordinates robot1) (6.8 16.5))
        ((robot.coordinates robot0) (7.8 16.5))
        ((robot.rotation robot1) 0)
        ((machine.progress_rate machine1) 0)
    )

    (def-static-facts
        ((machine.instance machine0) machine)
        ((machine.processes_list machine5)(6))
        ((machine.processes_list output_machine0) nil)
        ((machine.type output_machine0) output_machine)
        ((machine.coordinates_tile output_machine0) (27 18))
        ((machine.coordinates_tile machine3) (25 9))
        ((machine.instance machine5) machine)
        ((machine.instance input_machine0) machine)
        ((machine.coordinates machine1) (25.5 3.5))
        ((package.all_processes package0) ((3 1))); (1 3) (2 6) (4 7) (6 3) (5 6)))
        ;((package.all_processes package1) ((1 2) (3 6) (2 3) (6 4) (5 5) (4 4)))
        ((package.instance package0) package)
        ((machine.coordinates_tile input_machine0) (0 7))
        ((machine.coordinates machine2) (15.5 6.5))
        ((machine.type input_machine0) input_machine)
        ((machine.type machine3) standard_machine)
        ((machine.instance machine1) machine)
        ((machine.type machine5) standard_machine)
        ((machine.coordinates_tile machine2) (15 6))
        ((machine.instance machine3) machine)
        ((machine.coordinates machine5) (20.5 13.5))
        ((machine.processes_list machine1) (2))
        ((machine.instance machine2) machine)
        ((machine.type machine1) standard_machine)
        ((machine.coordinates_tile machine1) (25 3))
        ((machine.coordinates machine4) (9.5 11.5))
        ((machine.type machine4) standard_machine)
        ((machine.processes_list machine4) (5))
        ((machine.processes_list machine2) (3))
        ((machine.processes_list machine3) (4))
        ((machine.instance output_machine0) machine)
        ((machine.type machine0) standard_machine)
        ((machine.processes_list input_machine0) nil)
        ((machine.coordinates input_machine0) (0.5 7.5))
        ((machine.instance machine4) machine)
        ((machine.coordinates output_machine0) (27.5 18.5))
        ((machine.coordinates_tile machine4) (9 11))
        ((machine.coordinates_tile machine5) (20 13))
        ((machine.coordinates machine3) (25.5 9.5))
        ((machine.type machine2) standard_machine)
        ((machine.coordinates machine0) (8.5 3.5))
        ((machine.coordinates_tile machine0) (8 3))
        ((machine.processes_list machine0) (1))
                )
)

