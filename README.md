# Creating a Rust Websocket Server with Timely Dataflow

🎵 *Currently playing: Rolbac - Mystical says “In Silence you will hear stillness and black hours”* 🎵

As I set up this modest websocket server to get the rust off my Rust, I started remembering why I enjoyed working with it so much. Coming from writing a lot of Scala, Javascript, and .NET code, Rust has a feel to it that is quite hard to describe. It's fun. It feels powerful. The compiler messages are so… containing? More so than any other compile messages I’ve come across in 6 years. Pairing these detailed and concise messages with ChatGPT and, of course, TFM, building this proof of concept was an extremely fun and enlightening experience.

**Motivation:** As my main portfolio project progresses, I am in need of a way to visualize financial asset data in a more insightful way. I had used timely dataflows in the past to build PoCs, so I’m aware of their power to give life to these streams of data in an elegant, really-hard-to-grasp-at-first way.

But bear with me, we’ll go through it and soon you too will be creating dataflows for your streams!

Think of dataflows as extremely efficient pipelines that you can use to transform and process your data. More poetically speaking, you can imagine boring strings flying into a tube where countless hands, each with a specific intent and purpose, do “stuff” on these chains of characters and on the other side, what you get is a juicy, valuable insight. Dataflows allow us to create pipelines with extendable powers like this easily. In this case, I wanted to be able to consume massive amounts of data about the crypto markets (at the time of writing, BTC just hit 45k again. An alt season is coming and market sentiment analysis tools can actually be game changers in these times). So yes, I’m setting up an alert system for my telegram, partially por el amor al arte, partially because there is true value hidden behind these magical character chains we’ll be subscribing to…

---

**SO… what are we doing?** We will create a websocket server that will connect to the websocket endpoint provided by the Polygon platform. This websocket features different kinds of events that we will subscribe to, process with timely dataflows, and turn into rich, insightful information.

**The core components of this application include:**
- Subscription and deserialization of websocket messages.
- Channeling of parsed data into dataflows.
- Dynamic dataflow creation for particular events.
- Dynamic data filter application.
- Dynamic alert system.

---

## Subscription and Deserialization

If you have not worked with websockets before, you can think of them as hoses of data that allow your application to very easily begin to be flooded with messages. These messages represent events that the server you are connecting to is ready to distribute with its consumers, people like you and me who are thirsty for data.

So, what we do to subscribe is use X crate to send messages to the Polygon server to:
1. Connect
2. Authenticate
3. &4, N - Subscribe to the events we are interested in.

Now, we need to know the shape that the incoming messages have; otherwise, our application will have no clue what to do with all strings. They are just characters! The deserialization step involves transforming strings into structures that our system understands.

---
## Channeling

---
## Dynamic Dataflows

---

## Dynamic Filters

The main motivation behind this functionality is to allow users, like you (and me, I use this stuff), to create the filters they want. As a user, all you need to know is:
- The structure of the data being filtered.
- The type of operation that you want to perform on the data structure to filter it.

With this in mind, we’ll create a filtering structure that will demand knowing the types being filtered and which operations to apply.

--- 
## (In progress) Alerts
    Currently working on adding alerts with telegram
---
