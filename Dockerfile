FROM public.ecr.aws/lambda/python:3.12

RUN dnf install -y gcc-c++

# For -fsanitize=undefined and -fsanitize=address
RUN dnf install -y libasan libubsan

# Precompile bits/stdc++.h: https://gcc.gnu.org/onlinedocs/gcc/Precompiled-Headers.html
# I believe flags like -Wall are ignored, but flags like -std, -O2, and -fsanitize=address must
# match the flags used to precompile the header.
RUN mkdir -p /precompiled-headers/bits/stdc++.h.gch
RUN g++ -std=c++11 -O2 -o /precompiled-headers/bits/stdc++.h.gch/01 /usr/include/c++/11/x86_64-amazon-linux/bits/stdc++.h
RUN g++ -std=c++17 -O2 -o /precompiled-headers/bits/stdc++.h.gch/02 /usr/include/c++/11/x86_64-amazon-linux/bits/stdc++.h
RUN g++ -std=c++23 -O2 -o /precompiled-headers/bits/stdc++.h.gch/03 /usr/include/c++/11/x86_64-amazon-linux/bits/stdc++.h
RUN g++ -std=c++11 -O2 -fsanitize=address -o /precompiled-headers/bits/stdc++.h.gch/04 /usr/include/c++/11/x86_64-amazon-linux/bits/stdc++.h
RUN g++ -std=c++17 -O2 -fsanitize=address -o /precompiled-headers/bits/stdc++.h.gch/05 /usr/include/c++/11/x86_64-amazon-linux/bits/stdc++.h
RUN g++ -std=c++23 -O2 -fsanitize=address -o /precompiled-headers/bits/stdc++.h.gch/06 /usr/include/c++/11/x86_64-amazon-linux/bits/stdc++.h

RUN dnf install -y java-21-amazon-corretto-devel
RUN dnf install -y time

COPY target/lambda/online-judge-rust/bootstrap ${LAMBDA_RUNTIME_DIR}/bootstrap

# This is passed in as argv[1] to /var/runtime/bootstrap. The value shouldn't matter.
CMD [ "_handler" ]
