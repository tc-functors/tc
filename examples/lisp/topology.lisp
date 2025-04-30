
(route my-route "/api/foo")
(function foo-fn "../foo")
(event my-event "adhoc")
(channel my-channel)
(queue my-dlq)

(compose my-route foo-fn my-event)

(defun fact (n)
  (if n <= 0
      1
      n * (fact n - 1)))

(define n 4)

(define compose (lambda (f g) (lambda (x) (f (g x)))))

(define inc (lambda (x) (+ x 1)))
(define dec (lambda (x) (- x 1)))

(do
 (println (* n 3))
 (println (format "{testing} {} {}!" (fact 10) (eval test)))
 (inc 10)
 (env))
