# MiZe
- An implementation of how I think data should be handled. 
- Slogan: Elevate the unix file Philosophy into the age of connectivity.

## Why I'm doing this.
- What annoyes me about Software these days is things are done in so many different ways.
	- e.g: Different Cloud Apps for syncing data (Google-Cloud, Onedrive, Dropbox)
	- e.g: Every App stores the Data it deals with in it's own way (the way that the developer thought is best for this particular app)

- Back when Unix came out EVERYTHING was stored in one place: the file system of the computer. 
	- (just as simple bytestreams/files, organized by a folder-structure)
	- and NO program stored and synced any data around in unpredictable ways.

- This Philosophy is still used in every Unix/Linux machine, on a local level, but completely abstracted away (especially on mobile phones) from the user (by every App).

- I would say: The Unix File system just hasn't evolved into the age of connectivity. It is still completely the same. And I think it shouldn't be.

- miZe aims to bring that idea of storing all data
	- in one place
	- in one predictable and simple way
	- and programms only taking data from there and not storing it everywhere in messy ways
- back ... into the age of connectivity.

- That's why the slogan is going to be: Elevate the unix file Philosophy into the age of connectivity.

## How I'm going to do this.
- Is not yet completely clear, but what I know so far:
	- Using what I call "Items" instead of Files
		- With "Items" being a key-value store of strings/bytestreams
		- Those "Items" have types, that give meaning to the key-value pairs.
	- A Server Written in Rust that stores those "Items".
	- A way to mount "Items"(or parts of them) into a regular filesystem.
	- Webcomponents to show/render the contents of Items in a Browser (or electron style apps).
	- Server middleware to connect external apis (emails, youtube, google account) to this ecosystem.
	- The Server is one binary without any external requirements (that should run on any POSIX system)
		- to make it simpler to deploy
		- I've previously experimented with using MongoDB to store items, but then there has to be a MongoDB instance running that the server can connect to. And the connection can fail ... and lots of other things can go wrong.
		- I want a binary that you just start and it's a working server.
