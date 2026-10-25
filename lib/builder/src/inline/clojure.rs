use kit as u;

fn gen_deps_edn(dir: &str) {
    let data = format!(
r#"
{{:deps
 {{org.clojure/clojure {{:mvn/version "1.12.6"}}
  org.clojure/data.json {{:mvn/version "2.5.2"}}
  com.amazonaws/aws-lambda-java-core {{:mvn/version "1.2.3"}}}}
 :aliases
 {{:build {{:deps {{io.github.clojure/tools.build {{:mvn/version "0.10.0"}}}}
          :ns-default build}}}}}}
"#);
    let f = format!("{}/deps.edn", dir);
    u::write_str(&f, &data);
}

fn gen_builder(dir: &str) {
    let data = format!(
r#"
(ns build
  (:require [clojure.tools.build.api :as b]))

(def class-dir "target/classes")
(def uber-file "lambda.jar")

(def basis (delay (b/create-basis {{:project "deps.edn"}})))

(defn clean [_]
  (b/delete {{:path "target"}}))

(defn uber [_]
  (clean nil)
  (b/copy-dir {{:src-dirs ["." "resources"]
               :target-dir class-dir}})
  (b/compile-clj {{:basis @basis
                  :ns-compile '[lambda]
                  :class-dir class-dir}})
  (b/uber {{:class-dir class-dir
           :uber-file uber-file
           :basis @basis
           :main 'lambda}}))
"#);
    let f = format!("{}/build.clj", dir);
    u::write_str(&f, &data);
}

fn gen_wrapper(dir: &str) {
    let f =  format!(
        r#"
(ns lambda
  (:gen-class
   :implements [com.amazonaws.services.lambda.runtime.RequestStreamHandler])
  (:require
   [handler :as handler]
   [clojure.data.json :as json]
   [clojure.java.io :as io]))

(defn -handleRequest [this is os context]
  (let [w (io/writer os)]
    (-> (json/read (io/reader is) :key-fn keyword)
        (handler/handler)
        (json/write w))
    (.flush w)))
"#
    );

    let wrapper = format!("{}/lambda.clj", dir);
    u::write_str(&wrapper, &f);
}

pub fn gen_dockerfile(dir: &str, _pre: &Vec<String>) {
    let clojure_version = "1.10.3.1029";
    gen_deps_edn(dir);
    gen_builder(dir);
    gen_wrapper(dir);
    let f = format!(
        r#"
FROM maven:3-amazoncorretto-21 AS builder

WORKDIR /build
COPY . .

RUN curl -O https://download.clojure.org/install/linux-install-{clojure_version}.sh && \
  chmod +x linux-install-{clojure_version}.sh  && \
  ./linux-install-{clojure_version}.sh && \
   rm -f linux-install-{clojure_version}.sh

RUN clojure -T:build uber
RUN rm -rf target lambda.clj deps.edn build.clj
"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}
