(ns build
  (:require [clojure.tools.build.api :as b]))

(def lib 'example/handler)
(def version (format "1.2.%s" (b/git-count-revs nil)))
(def class-dir "target/classes")
(def uber-file (format "target/lambda.jar" (name lib) version))

;; delay to defer side effects (artifact downloads)
(def basis (delay (b/create-basis {:project "deps.edn"})))

(defn clean [_]
  (b/delete {:path "target"}))

(defn uber [_]
  (clean nil)
  (b/copy-dir {:src-dirs ["." "resources"]
               :target-dir class-dir})
  (b/compile-clj {:basis @basis
                  :ns-compile '[handler]
                  :class-dir class-dir})
  (b/uber {:class-dir class-dir
           :uber-file uber-file
           :basis @basis
           :main 'handler}))
