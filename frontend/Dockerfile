FROM node:latest

WORKDIR /usr/src/opendatacapture
COPY . .

RUN npm install
RUN npm run build

FROM node:latest

COPY --from=0 /usr/src/opendatacapture/build ./build

RUN npm install -g serve

CMD ["serve", "-s", "build"]
