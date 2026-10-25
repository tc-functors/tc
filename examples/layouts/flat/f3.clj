(defn process-event [event]
  (prn event))

(defn handler [event]
  (process-event event)
  event)
