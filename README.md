# Serverless Online Judge (Rust)

From the Competitive Programming Initiative. A successor to https://github.com/cpinitiative/online-judge.

## Goal

To create a low-cost, reliable, and fast online judge that:

- Supports C++, Java, Python, and possibly other languages
- Can be used to run code against many test cases in parallel
- Can be extended to support graders, scorers, and interactive problems

Notably, the following are not goals of this project:

- *Is not necessarily consistent*. This is because AWS Lambda can run on different CPU architectures. Since USACO problems generally aren't too sensitive to time constraints, we are OK with this.
- *Is not necessarily secure*. Malicious code will not harm other AWS resources, but could theoretically return falsified results.

This online judge is meant to be used with the USACO Guide IDE or USACO Guide Groups, so the experience is optimized to make honest users happy most of the time rather than catch malicious users (i.e. we would rather grade problems faster even if that means malicious users can access expected output).

## Development

Install Rust, Cargo, and project depenencies (notably [`cargo-lambda`](https://github.com/cargo-lambda/cargo-lambda)).

### Running in development

```
cargo lambda watch -P 9001
``` 

And POST `http://localhost:9001/compile-and-execute`.

### Deploying

Continuous deployment is set up with Github Actions; all you need to do is push to main.

### Miscellaneous Commands


To test base docker image (useful for determining what packages need to be installed):

```
docker run --rm -it --platform linux/amd64 --entrypoint /bin/bash public.ecr.aws/lambda/python:3.12
```

To build and test dockerfile locally:

```
cargo lambda build
docker build --platform linux/amd64 -t oj-rust .
docker run --platform linux/amd64 -p 9000:8080 oj-rust

# or:
docker run -it --platform linux/amd64 -p 9000:8080 --entrypoint /bin/bash oj-rust

# to send command:
curl "http://localhost:9000/2015-03-31/functions/function/invocations" -d '{}'

# to get format for ^: https://github.com/brefphp/local-api-gateway/blob/main/src/apiGateway.ts
```

To upload to ECR:

```
# Login to AWS ECR
aws ecr get-login-password --region us-east-1 | docker login --username AWS --password-stdin 417600709600.dkr.ecr.us-east-1.amazonaws.com

# Tag the image
docker tag oj-rust 417600709600.dkr.ecr.us-east-1.amazonaws.com/online-judge-rust:latest

# Push
docker push 417600709600.dkr.ecr.us-east-1.amazonaws.com/online-judge-rust:latest
```

To deploy lambda for the first time:
- Create lambda through AWS console
- Add a function URL with CORS
- Set timeout to 30 seconds
- Set memory to 1769 MB (1 vCPU)

### Todo

Note that there are actually two CPUs -- AMD and Intel -- so timings are not consistent. With this, I saw 1.93 and also 1.30:

```
double x; cin >> x; int y = x; for (int i = 0; i < y; i++) x += sqrt(x); cout << x << endl;
```

x = 200000000 input.

```js
for (let i = 0; i < 100; i++) fetch("https://v3nuswv3poqzw6giv37wmrt6su0krxvt.lambda-url.us-east-1.on.aws/compile-and-execute", {
     method: "POST", 
    headers: {"Content-Type": "application/json" }, body: JSON.stringify({compile:{
    "source_code": `// Source: https://usaco.guide/general/io

#include <bits/stdc++.h>
using namespace std;

int main() {
double x; cin >> x; int y = x; for (int i = 0; i < y; i++) x += sqrt(x); cout << x << endl;;
}

`,
    "compiler_options": "-O2 -std=c++17",
    "language": "cpp"
},execute:{stdin:"200000000", timeout_ms:5000}}) }).then(x => x.json()).then(x => console.log(x.execute.stderr.match(/wall clock.*/)[0]))
```

Should benchmark this to determine how off the timings are / whether we can just add a multiplicative factor to it.


We can also get improved error messages. This is an example of what Leetcode tells you:


```
/*
Line 15: Char 8: runtime error: signed integer overflow: 2147483647 + 2147483647 cannot be represented in type 'int' (solution.cpp)
SUMMARY: UndefinedBehaviorSanitizer: undefined-behavior solution.cpp:15:8
*/
```

We should have better logging: https://docs.aws.amazon.com/lambda/latest/dg/rust-logging.html

We should add tests. Here's an utf8 encoding regression test:

```
#include <iostream>
using namespace std;

int main() {
	char c = 128;
	cout << c << endl; // this shouldn't crash the lambda funciton
}
```
