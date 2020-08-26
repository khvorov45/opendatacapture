FROM node:latest

WORKDIR /usr/src/opendatacapture
COPY . .

RUN npm install

RUN npm install -g serve

ENV REACT_APP_ODC_API_ROOT http://localhost:4321
CMD npm run build && serve -s build
