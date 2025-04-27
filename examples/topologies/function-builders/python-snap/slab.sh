#!/bin/sh
set -e -u -o pipefail

# pip download -vv --platform manylinux2014_x86_64 --only-binary=:all: --dest /build/python \
#     aws-lambda-powertools \
#     aws-xray-sdk \
#     honeybadger \
#     opencv-python-headless \
#     matplotlib \
#     transformers \
#     pillow \
#     wrapt \
#     tabulate \
#     nltk \
#     urllib3 \
#     pandas \
#     xgboost \
#     gensim \
#     scikit-learn \
#     spacy \
#     backoff \
#     psycopg2-binary \
#     ultralytics \
#     fasttext-wheel \
#     torch==2.3.0 \
#     torchvision==0.18.0

pip download -vv --platform manylinux2014_x86_64  --only-binary=:all: --dest /build transformers==4.44.0 numpy pandas xgboost

pip download --only-binary=:all: --dest /build torch==2.3.0 torchvision==0.18.0 --index-url https://download.pytorch.org/whl/cpu
