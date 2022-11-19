# fula-pool
This is the blockchain (substrate) code that handles the pools in Fula.

# Actions

It enables users to do the following actions:

1- Create Pool: Anyone can create a pool which is open to others to join

2- Request to Join a Pool: Anyone can request to join an already created pool. For the join request to be accepted, more than half of the current pool members must approve the request

3- Approve/Reject join requests: Current pool members can either accept or reject join requests. Accepts and rejects can be made based on a set of agreed-on criteria. In Fula, the criteria are the ping time. Each member pings the person who wants to join and if they meet ping time criteria, the request is accepted.


## Notes

A few points:
- Pool creator does not have any power over other members and is just the person who initiates a pool and set the criteria. However, in Fula, pool creator must stake tokens to be able to create a pool.
- Each pool has a limit on the number of people who can join. If more people send join requests, a copy of the pool, with the exact same criteria is created by the contract, and the first 3 join requests must be approved by the original pool members. After 3 members join the copy pool, it becomes an independent pool and join requests then must be approved by its members.
