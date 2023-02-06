(begin
    (def-lambda go_random (lambda (?r ?l ?u)
                            (let ((x (rand-int-in-range ?l ?u))
                                  (y (rand-int-in-range ?l ?u)))
                                  (navigate_to ?r x y))))

    (def-lambda
        find_machines_for_process
            (lambda (?process)
                (begin
                    (define __process__
                        (lambda (?px seq)
                            (if (null? seq)
                                nil
                                (if (contains (machine.processes_list (car seq)) ?px)
                                    (cons (car seq) (__process__ ?px (cdr seq)))
                                    (__process__ ?px (cdr seq))))))
                    (define machines (instances machine))
                    (define result (__process__ ?process machines))
                    result)))


    (def-lambda find_output_machine 
        (lambda nil
            (begin 
                (define __lambda__ 
                    (lambda (seq)
                        (if (null? seq)
                            nil
                            (if (= (machine.type (car seq)) output_machine)
                                (car seq)
                                (__lambda__ (cdr seq))))))
                (__lambda__ (instances machine)))))

    (def-lambda take_first 
        (lambda (seq)
            (if (null? seq)
                nil
                (cons (caar seq) (take_first (cdr seq)))))))