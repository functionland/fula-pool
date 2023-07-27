# fula-pool
This is the blockchain (substrate) code that handles the pools in Fula. Pools are a set of nodes that join together to share resources with each other or provide resources to others.

[//]: # (#SBP-M1 review: needs more context - e.g. which smart contract, including link, how are resources limited)
It limits the amount of CPU and storage needed for the smart contract and enable the chain to do hierarchical consensus.

# Actions

It enables users to do the following actions:

[//]: # (#SBP-M1 review: use proper markdown 1. 2. 3.)
1- Create Pool: Anyone can create a pool which is open to others to join

2- Request to Join a Pool: Anyone can request to join an already created pool. For the join request to be accepted, more than half of the current pool members must approve the request

3- Approve/Reject join requests: Current pool members can either accept or reject join requests. Accepts and rejects can be made based on a set of agreed-on criteria. In Fula, the criteria are the ping time. Each member pings the person who wants to join and if they meet ping time criteria, the request is accepted.


## Notes

A few points:

[//]: # (SBP-M1 review: no token staking currently implemented for pool creation)
- Pool creator does not have any power over other members and is just the person who initiates a pool and set the criteria. However, in Fula, pool creator must stake tokens to be able to create a pool.

[//]: # (SBP-M1 review: copy pool functionality not implemented)
[//]: # (SBP-M1 review: how is the contract calling this pallet to create the pool?)
- Each pool has a limit on the number of people who can join. If more people send join requests, a copy of the pool, with the exact same criteria is created by the contract, and the first 3 join requests must be approved by the original pool members. After 3 members join the copy pool, it becomes an independent pool and join requests then must be approved by its members.

[//]: # (SBP-M1 review: no tests)
[//]: # (SBP-M1 review: no benchmarks)
[//]: # (SBP-M1 review: no description of dispatchable functions)
[//]: # (SBP-M1 review: add section with terminology - e.g. https://github.com/paritytech/substrate/blob/master/frame/assets/README.md)