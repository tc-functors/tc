#!/bin/sh
exec /usr/local/bin/aws-lambda-rie /var/lang/bin/python3.10 -m awslambdaric $@

