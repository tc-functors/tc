(import spork/http)
(import spork/json)

(defn rapi []
  (os/getenv "AWS_LAMBDA_RUNTIME_API"))

(defn make-url [path]
  (string "http://" (rapi) "/2018-06-01/runtime/invocation" path))

(defn get-request []
  (let [request (http/request :GET (make-url "/next"))]
    {:request-id (get-in request [:headers "lambda-runtime-aws-request-id"])
     :body (json/decode (get request :body))}))

(defn send-response [request-id body]
  (def url (make-url (string "/" request-id "/response")))
  (def res (http/request
	    :POST
	      (make-url (string "/" request-id "/response"))
	      :body (json/encode body)
	      :headers {:content-type "application/json"}))
  res)

(defn send-error [request-id error-body]
  (http/request :POST
     (make-url (string "/" request-id "/error"))
     error-body
     :content-type "application/json"))

(defn handle-request [request handler-fn]
  (let [request-id (get request :request-id)
        body (get request :body)]
    (->> (handler-fn body)
	 (send-response request-id))))

(defn handle [handler-fn]
  (if (rapi)
    (-> (get-request)
	(handle-request handler-fn))))
