# tablehop

This is a hacked together OpenTable reservation bot.
I was familiar with the concept of sneaker/ticket/reservation bots and the like, but didn't know much about how they worked, exactly.
A free first-come first-serve tasting menu reservation for a restaurant's 10th anniversary came up and it felt like the perfect opportunity to scramble to the finish line and pick up some knowledge.

This bot is specific to reserving experiences not standard table reservations.
As experiences require a credit card, this bot also complete's checkout

Things I learned:
- Scheduling in Rust (more of a pain than I thought + some outdated not well documented libraries in use here)
- To avoid Selenium and the like for speed gains. No automateed browsers.
- Packet injection (I believe it's called now) - how most botters do it these days.
- Challenges - didn't really run into any but learned about them.
- Reverse engineering OpenTable's API endpoints. Bad behavior. Actually probably the most significant skill. + fun!!
- Pulling ids and what not from the site html (sadly GQL doesn't provide everything you need to properly book so scraping is necessary, at least  on startup)
- Working with OpenTable's GQL endpoint - requesting and parsing data so you can find available slots.
- Checkout with credit card. Requires obtaining payment session + making requests to the necessary endpoints you dig up and so on. Also, bad behavior.
- Login and using the appropriate session cookies. (Don't wanna get flagged and rejected - though Opentable seems to be pretty lax)

