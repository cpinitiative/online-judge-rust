FROM public.ecr.aws/lambda/python:3.12

RUN dnf install -y gcc-c++

# For -fsanitize=undefined and -fsanitize=address
RUN dnf install -y libasan libubsan

RUN dnf install -y java-21-amazon-corretto-devel
RUN dnf install -y time tar

COPY target/lambda/online-judge-rust/bootstrap ${LAMBDA_RUNTIME_DIR}/bootstrap

# This is passed in as argv[1] to /var/runtime/bootstrap. The value shouldn't matter.
CMD [ "_handler" ]
