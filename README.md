# MiZe
- An implementation of how I think data should be handled. 
- Slogan: _Elevate the Unix file philosophy into the age of connectivity._

## Why am I doing this?
- What annoys me about software these days is, that things are done in so many different ways.
	- e.g: Different cloud applications for syncing data (Google Cloud, OneDrive, Dropbox)
	- e.g: Every app stores the data it deals with in it's own way (the way that the developer thought is best for this particular app)

- Back when Unix came out **everything** was stored in one place: the file system of the computer. 
	- (just as simple bytestreams/files, organized by a folder-structure)
	- and NO program stored and synced any data around in unpredictable ways.

- This philosophy is still used in every Unix/Linux machine, on a local level, but completely abstracted away (especially on mobile phones) from the user (by every app).

- I would say: The Unix file system just hasn't evolved into the age of connectivity. It is still completely the same. And I think it shouldn't be.

- MiZe aims to bring that idea of storing all data
	- in one place
	- in one predictable and simple way
	- and programs only taking data from there and not storing it everywhere in messy ways

- That's why the slogan is going to be: _Elevate the unix file philosophy into the age of connectivity._

## How am I going to do this?
Is not yet completely clear, but here's what I know so far:
- Using what I call "Items" instead of files
	- With "Items" being a key-value store of strings/bytestreams
	- Those "Items" have types, that give meaning to the key-value pairs.
- A server written in Rust that stores those "Items".
- A way to mount "Items" (or parts of them) into a regular filesystem.
- Web Components to show/render the contents of items in a browser (or Electron-style apps).
- Server middleware to connect external APIs (e.g. emails, YouTube, Google accounts) to this ecosystem.
- The server is one binary without any external requirements (that should run on any POSIX system)
	- This makes it very simple to deploy
	- I've previously experimented with using MongoDB to store items, but that requires a MongoDB instance to always be running that the server can connect to. And the connection can fail ... and lots of other things can go wrong.
	- I want a binary that you can simply start, which then provides a working server.
