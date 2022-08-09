# **Pallet Chat**

## **About**

Communication has always been an integral part of any process. And for making the process interactive and simpler between the customer and the worker, a feature for on chain communication needed to be established. For this very reason, the chat pallet was created. The chat pallet focuses on a 1-1 message sending and responding service. It is developed as a mode of communication between the customer and the worker. The chat feature between any two parties is activated when a task is created and remains in service throughout the lifespan of the created task. The service becomes inactive when the task is completed.

## **Roadmap**

* Extrinsic to create message on chain from sender to receiver. We have used Storage as StorageValue & StorageMap. Going with StorageMap.
* Extrinsic to reply to a message on chain from receiver to sender. We have used Storage as StorageValue & StorageMap. Going with StorageMap.
* Extrinsic to mark a message as read by the sender after the receiver has replied.		
* Various error handlers according to logic for the extrinsics.	
* Various conditions for displaying on successful execution according to the objective of the extrinsic.		
* Configuring the mock runtime for integrating tests for all the completed extrinsics.

## **Feature Flow**
![pallet_chat drawio](https://user-images.githubusercontent.com/57192661/155514337-0638b59d-cb05-48b1-a4f9-3730e65e5f25.png)

## **Additional**
The chat pallet can be used for various on chain applications and can be used for multiple use cases.
