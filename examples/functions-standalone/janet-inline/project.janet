(declare-project
 :name "lambda"
 :description "Janet example"
 :author "icylisper"
 :license "MIT"
 :dependencies ["spork"])

(declare-executable
 :name "bootstrap"
 :source ["main.janet"]
 :entry "main.janet")
