FROM node:latest

WORKDIR /usr/src/opendatacapture
COPY . .

RUN npm install

ARG API_ROOT=api
ENV REACT_APP_ODC_API_ROOT ${API_ROOT}
RUN npm run build

RUN npm install -g serve
CMD serve -s build
