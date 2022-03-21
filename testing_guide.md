# Setup: Testing Guide
First, complete the basic Rust setup instructions. If you want to play at the code level. \
If no, then please follow this simple guide to make your life easy :+1:

## Install Docker & Docker-Compose

First, install Docker and Docker Compose. Follow the basic installation guide for [Docker](https://docs.docker.com/engine/install/) and [Docker Compose](https://docs.docker.com/compose/install/)

For a Windows Machine : [Follow the guide mentioned here](https://docs.docker.com/desktop/windows/install/)

Installation from the terminal on a Ubuntu / Linux machine 

```shell
$ curl -L "https://github.com/docker/compose/releases/download/1.29.2/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
```
```shell
$ chmod +x /usr/local/bin/docker-compose
```

## Docker guide

Clone the repo for Phase 2 [ Milestone 1 ] [Dot Marketplace Docker](https://github.com/WowLabz/dot_marketplace_docker/tree/Phase2_Milestone1)

```shell
# To check the compose version
$ docker-compose --version
```

```shell
# To run the build
$ docker-compose up --build -d
```

```shell
# To stop the service
$ docker-compose down
```

```shell
# To view the installed images locally
$ docker images
```

```shell
# To delete the images
$ docker rmi <IMAGE ID>
```

>After building the image, you can also view it on polkadot.js explorer via local node

<br>

## Functional Guide for Dot Marketplace

* `Customer` Workflow:
    1. Create Task
    2. Approve Task
    3. Provide Worker Ratings
    4. Can communicate using chat
    5. Disapprove Task
    6. Disapprove Rating
    7. Raise Dispute

* `Worker` Work Flow:
    1. Bid For Task
    2. Complete Task
    3. Provide Customer Ratings
    4. Can communicate using chat
    5. Disapprove Rating
    6. Raise Dispute

* `Juror` Work Flow:
    1. Accept Jury Duty
    2. Cast Vote

<br>

## Migration from V1 to V3

* >Video  walkthrough \ 
The video demonstrates the flow of tasking backend which was created as a part of Phase 1 but has now been upgraded to substrate frame v3 \
[Video demo in with polkadot.js explorer connected to the node](https://user-images.githubusercontent.com/57192661/159009199-51befb8b-64d7-4b43-b10f-8324d43fd675.mp4)

## Chat app workflow testing on Polkadot.js explorer 

* >Video  walkthrough \
A functional demo of the chat system flow \
[Video demo in with polkadot.js explorer connected to the node](https://user-images.githubusercontent.com/58659064/158811706-868510e4-dfdd-42d0-8d2e-9620a59e141c.mp4)

## Decentralized Court workflow testing on Polkadot.js explorer 

* >Video  walkthrough \
A functional demo of the court system when a publisher disapproves a task\
[Video demo in with polkadot.js explorer connected to the node](https://user-images.githubusercontent.com/58659064/159296018-f6cffc16-0fed-46b8-9c1b-142d04f45974.mp4)
<br>

To read about the working of the pallet please refer the guide [Description](https://github.com/WowLabz/dot-marketplace-v2/blob/main/README.md)


