# ktkbot
Send push notifications when new tennis events are put up for Kløvermarkens Tennis Klub.

## TODO
- [x] Fetch page.
- [x] Parse events.
- [x] Identify new events and update local list of events accordingly.
- [x] Fetch all events including the ones that don't show up on the page until you scroll.
- [x] Loop infinitely or until terminated.
- [x] Handle errors properly.
- [x] Logging.
- [x] Send push notifications.
- [x] Cache events in memory.
- [x] Sort events by date before outputting.
- [x] Recognize error responses from Pushover.
- [x] Write events to file in compact json instead of pretty json.
- [ ] Improve event parsing implementation.
  - Meta information such as number of free spots in an event should not be included in comparisons. Instead of doing deep event comparisons, it may be possible to extract an ID for each event from the HTML and compare by that. Another solution would be to properly parse all fields.
- [ ] Command line arguments for configuration.
- [ ] Asynchronous fetching and termination.
- [ ] Improve error handling.
- [ ] Documentation.

