# Creating a Rust Websocket Server with Timely Dataflow

**Motivation:** As my main portfolio project progresses, I am in need of a way to visualize financial asset data in a more insightful way. I had used timely dataflows in the past to build PoCs, so I’m aware of their power to give life to these streams of data in an elegant, really-hard-to-grasp-at-first way.

But bear with me, we’ll go through it and soon you too will be creating dataflows for your streams!

Think of dataflows as extremely efficient pipelines that you can use to transform and process your data. You can imagine boring strings flying into a tube where countless hands, each with a specific intent and purpose, do “stuff” on these chains of characters and on the other side, what you get is valuable insight. 
Dataflows allow us to create pipelines with extendable powers like this easily. In this case, I wanted to be able to consume massive amounts of data about the crypto markets (at the time of writing, BTC just hit 45k again. An alt season is coming and market sentiment analysis tools can actually be game changers in these times). So yes, I’m setting up an alert system for my telegram, partially por el amor al arte, partially because there is true value hidden behind these magical character chains we’ll be subscribing to…

---

**SO… what are we doing?** We will create a websocket server that will connect to the websocket endpoint provided by the Polygon, Binance, Alchemy, Moralis (and more coming) platforms. This websocket feature different kinds of events that we will subscribe to, process with timely dataflows, and turn into rich, insightful information.

**The core components of this application include:**
- Subscription and deserialization of websocket messages.
- Channeling of parsed data into dataflows.
- Dynamic data filter application.
- Dynamic alert system.

---

## Subscription and Deserialization

If you have not worked with websockets before, you can think of them as hoses of data that allow your application to very easily begin to be flooded with messages. These messages represent events that the server you are connecting to is ready to distribute with its consumers, people like you and me who are thirsty for data.

So, what we do to subscribe is use the ```tokio-tungstenite``` crate to send messages to the websocket servers to:
1. Connect
2. Authenticate
3. &4, N - Subscribe to the events we are interested in.

Now, we need to know the shape that the incoming messages have; otherwise, our application will have no clue what to do with all these strings. They are just characters! The deserialization step involves transforming strings into data structures that our system understands. For this, we use the ```serde-json``` crate in the ```src/subscriber/websocket_event_types.rs``` file to easily map each event to its corresponding model (```src/models/*```). 

---
## Channeling

Messages need to be sent to and from the websocket consumption thread, of ```async``` nature, into/out of dataflows, which run synchronously. We use crossbeam channels to achieve this:

```rust
use crossbeam_channel::{bounded, Sender};

let (sender, receiver) = bounded::<T>(n); // where T = the type of the messages this sender and receiver can handle (eg. PolygonEventTypes / BinanceEventTypes). 

// Continuously read from the channel and process messages
loop {
    match receiver.recv() {
        Ok(message) => {
        // Process the message
        // For example, update dataflows, broadcast to clients, etc.
        },
        Err(_) => {
        // Handle the case where the sender is dropped/disconnected
        break;
        }
    }
}
```

## Dynamic Filters

The main motivation behind this functionality is to allow end users, to create the filters they want. As a user, all you need to know is:
- The structure of the data being filtered.
- The type of operation that you want to perform on the data structure to filter it.

With this in mind, we’ll create a filtering structure that will demand knowing the types being filtered and which operations to apply.


There are examples of how you can set up your own filters on (for now) on startup.  
Moving forward, we will expose an API that will allow you to define and save your own filters,
which will then be applied to dataflows spawned on demand to process a particular set of topics. 

--- 
## (In progress) Alerts
    Currently working on adding alerts with telegram
---


---
## DB & Diesel

This project uses a relational timescale db, for which you need psql installed and have a PostgreSQL server running.

Write the first migration file in the migrations directory generated from running the  ```diesel setup``` command.
This directory is usually placed at the root of your project, along your src directory. 

If the migrations directory doesn't exist, you can create it manually. When you run ```diesel migration generate```,
it will create the migration file in this directory.

[[WIP]]When you add a migration to your Diesel project and rebuild your Docker image,
the migration file will be included in the Docker image. When the Docker container starts up, if it detects that there are pending migrations,
it will automatically apply them to the database.

Timescale is a postgres extension that helps us deal with time-series data. More on this coming soon.

Setup timescale - needs psql server :
```brew tap timescale/tap```
```brew install timescaledb```
```timescaledb-tune --quiet --yes```
```./timescaledb_move.sh```

We will be using Diesel as ORM to interface with our timescaledb. Define a DATABASE_URL in your .env. Remember that you also need to create a database and probably a user for your application, and a schema to keep things tidy. 
Execute this before running  the migrations. Check the samepl init-db.sql in the /migrations folder. Then, to set up diesel:

1. ```cargo install diesel_cli --no-default-features --features postgres```
2. ```diesel setup```
3. ```diesel migration run```


- Dockerized version: WIP

```docker exec -it rust-websocket-server-db-1 psql -U iriuser -d iridb```

```
docker-compose up db &&
docker exec -it rust-websocket-server-db-1 psql -U iriuser -d iridb
```
---
