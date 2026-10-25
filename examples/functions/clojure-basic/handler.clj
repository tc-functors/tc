(ns handler)

(defn process-event [event]
  (println "this is amazing")
  (prn event))

(defn handler [event]
  (process-event event)
  event)
