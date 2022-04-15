
(begin
  (def-objects
  '(t1 t2 truck)
  '(s l1 l2 l3 l4 l5 l6 l7 l8 e location))

  (def-initial-state
    (map '(
      ((at t1) s)
      ((at t2) s)
      ((connected s l1) yes)
      ((connected s l3) yes)
      ((connected l1 s) yes)
      ((connected l1 l2) yes)
      ((connected l1 l6) yes)
      ((connected l1 l8) yes)
      ((connected l2 l1) yes)
      ((connected l2 l4) yes)
      ((connected l2 l5) yes)
      ((connected l2 e) yes)
      ((connected l3 s) yes)
      ((connected l3 l6) yes)
      ((connected l4 l2) yes)
      ((connected l4 e) yes)
      ((connected l5 l2) yes)
      ((connected l5 e) yes)
      ((connected l6 l1) yes)
      ((connected l6 l2) yes)
      ((connected l6 l5) yes)
      ((connected l6 l7) yes)
      ((connected l7 l5) yes)
      ((connected l8 l2) yes)
      ((connected l8 l4) yes)
    )))

  (trigger-task t_move t1 e)
)