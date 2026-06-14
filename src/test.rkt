(define (fib n)
  (cond [(= n 0) 1]
        [(= n 1) 1]
        [else
          (+ (fib (- n 1)) (fib (- n 2)))]))

(define (addmult a b)
  (+ (* a a)
     (* b b)))

(define (map fn list)
  (cond [(empty? list) empty]
        [else
          (cons (fn (first list))
                (map fn (rest list)))]))

(define (dub x)
  (* x 2))
