FROM node:latest

WORKDIR /usr/src/opendatacapture
COPY . .

RUN npm install

RUN npm install -g serve

CMD npm run build && serve -s build
