(import spork/json)
(import ./runtime)

(defn handler [event]
  {:status "ok" :task "best" :from "spork"})

(defn main [& args]
  (runtime/handle handler))
