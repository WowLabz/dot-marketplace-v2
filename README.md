# **Dot Marketplace**

- Status: Open
- Proposer: Wow Labz
- Projects you think this work could be useful for: [Polkadot](https://polkadot.network/), [Kusama](https://kusama.network/), [Moonbeam](https://moonbeam.network/) and all Polkadot parachains/ parathreads 

### **Overview** 📄

Dot Marketplace is a general purpose decentralised marketplace created as a Substrate pallet.  

The current scope of work involves two user types: **Customer** and **Service Provider (or Worker)**

- The Customer can post a task and invite bids from Service Providers to fulfill it. 
- The Customer needs to deposit the budgeted amount in an escrow for the task to be published. 
- The Service Provider needs to deposit some token to participate in a bid. If not shortlisted, this bid amount is returned. 
- The Service Provider completes the task and submits it. 
- The Customer accepts the work and the escrowed amount is credited to the Service Providers wallet.
- The Customer rates the Service Provider and visa versa.

NOTE: If the Customer doesn't accept the work, a dispute is raised and it gets resolved in a decentralised court (out of current scope) which will be implemented in the next phase. 

The following diagrams highlight the workflow:


Customer                   
:-------------------------:
<img src = "https://user-images.githubusercontent.com/11945179/125753620-e1b094dc-552e-4a4f-9826-23cbefe3a677.png" widht=600>

Worker
:-------------------------:
<img src = "https://user-images.githubusercontent.com/11945179/125753635-1cc3170e-7a19-410e-a350-93f75a10e93f.png" widht=600>


Dot Marketplace is being built as a Substrate pallet. It would include boilerplate code that parachain teams can customize as per their own requirements. We believe this project has the potential to transform community participation, engagement and governance in decentralized projects.


### **Repository Hierarchy**
```
├── Dot Marketplace Network Node [link](https://github.com/WowLabz/tasking_backend)
│   ├── ./node ["Chainspecs for Node"]
│   ├── ./scripts [Packaging & Deployment Scripts]
│   ├── ./pallets/pallet-tasking [Pallets]
│   │	    └── ./pallet-tasking 
│   │    	        └── ./src/lib.rs [Tasking Pallet (being implemented)]
│   └── ./runtime [Runtime Module]
│    	    └── Included custom Tasking Pallet

```

The current focus is to enhance the existing Substrate pallet and allied code base to get a basic yet functional marketplace up and running:


### **Ecosystem Fit**

Dot Marketplace can be used by any decentralised project to float tasks and invite their community members to execute them for a reward. Its MVP was developed during the Polkadot India buildathon (2021).  

The inspiration for Dot Marketplace emerged from our own needs while building Yoda - a protocol that facilitates decentralised app development leveraging open data. Dot Marketplace would be used to create data, services and app marketplaces on Yoda, which would motivate us to maintain this project in the long run. 

![dotmarketplacegif](https://user-images.githubusercontent.com/11945179/124598936-c9f01000-de82-11eb-91d5-b2e37f1791df.gif)

### **List of Competitors**

Any product or services marketplace would qualify, here are some examples from outside the Polkadot ecosystem. 
1. [Human Protocol](https://data.iota.org/#/)
2. [Effect Network](https://www.snowflake.com/data-marketplace/)
3. [Ocean Protocol Market](https://market.oceanprotocol.com/)


## **Team** 👥

### **Team members**

* Amit Singh (product manager)
* Roshit Omanakuttan (technical architect)
* Varun Gyanchandani (backend lead)
* Loakesh Indiran (full stack dev)
* Siddharth Teli (backend dev)
* Ritiek Malhotra (backend dev)
* Bharath Kumar (tester)


### **Team Website**

- [http://www.wowlabz.com](https://www.wowlabz.com/) 

### **Project Website**
- Dot marketplace website is under construction

### **Legal Structure** 
- Indian, Private Limited Company 

Wow Labz

[Address](https://g.page/2gethr-ORR): Wow Labz, 2Gethr Cowork, Tower B, Mantri Commercio, Outer Ring Rd, near Sakra World Hospital, Kariyammana Agrahara, Bellandur, Bengaluru, Karnataka 560103

### **Team&#39;s experience**

Dot Marketplace is being built by the team at Wow Labz.
Wow Labz is one of India&#39;s leading turnkey product development companies.
Yoda Protocol has been conceptualised and is being built by the team at Wow Labz. The team has previously built a decentralised storage protocol called Lake Network - [https://lakenetwork.io/](https://lakenetwork.io/) in addition to multiple dApps on Ethereum, Stellar, EOS and Hyperledger.

A list of centralised apps published can be found [here](https://www.wowlabz.com/work/).


### **Team Code Repos**

* [https://github.com/orgs/WowLabz/projects](https://github.com/orgs/WowLabz/projects) 
* [https://github.com/WowLabz/tasking\_backend](https://github.com/WowLabz/tasking_backend)
* [https://github.com/WowLabz/tasking\_frontend](https://github.com/WowLabz/tasking_frontend)

### **Team LinkedIn Profiles (if available)**

Profiles of the people working actively on Dot Marketplace
* [Amit Singh](https://www.linkedin.com/in/startupamit/)
* [Roshit Omanakuttan](https://www.linkedin.com/in/roshit/)
* [Varun Gyanchandani](https://www.linkedin.com/in/varunsays/)
* [Loakesh Indiran](https://www.linkedin.com/in/loakesh-indiran-8a2282140)
* [Siddharth Teli](https://www.linkedin.com/in/siddharthteli/)
* [Ritiek Malhotra](https://www.linkedin.com/in/ritiek/)
* [Bharath Kumar](https://www.linkedin.com/in/bharath-kumar-h-13a572126/)

## **Development Roadmap**🔩

The development of Dot Marketplace is already underway. 
For the custom pallet (tasking) we have: 
1. Used various Substrate provided traits like - `Currency`, `ExistenceRequirement`, `LockIdentifier`, `LockableCurrency`, `ReservableCurrency` and few more;
2. Used the pre-existing pallets like `assets`, `balances` and `staking`;
3. Implemented custom structs like `TaskDetails` and `TransferDetails`. These in return are used for various functionalities like `create_task`, `bid_task`, `complete_task` and `approve_task`. A special transfer money function is only initiated once the task cycle gets completed and the escrow funds are released to the worker. 



### **Future Plans** 
Future releases of the Dot Marketplace include:

| Phase        | Deliverable   | Specification  |
| :-------------|:-------------:| :--------------|
| 2      | Decentralised Court | A fully decentralised dispute resolution mechanism along with configurible rules for slashing and reputation.          |
| 3      | Milestone based submissions | Making provisions to breakdown a project into multiple configurable milestones to allow parallel or sequential execution        |
| 4     | Decentralised Storage | Integration with IPFS or another decentralised storage platform        |

