FROM public.ecr.aws/lambda/python:3.12

COPY target/lambda/online-judge-rust/bootstrap /var/runtime/bootstrap

# This is passed in as argv[1] to /var/runtime/bootstrap. The value shouldn't matter.
CMD [ "_handler" ]
