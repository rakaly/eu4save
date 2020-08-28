## v0.2.0 - 2020-08-28

- Fix `Eu4Date::days_until` calculation involving September dates
- Add additional validation when constructing `Eu4Date` so that nonsensical dates can't be created
- `Eu4Date` now implements `Copy`
- Migrate memory map extraction to use anonymous region instead of temporary file
- Query engine reorganized to hide fields behind public methods to aid transparent caches
- Expose dlc function at root of docs
- Move data structs behind model module
- Include destroyed buildings in province building history

## v0.1.6 - 2020-08-24

- Include initial buildings in province building history query

## v0.1.5 - 2020-08-23

- Add province base tax, production, and manpower
- Start extracting province history
- Add province building history query to query engine

## v0.1.4 - 2020-08-20

- Fix province building deserialization for ironman saves

## v0.1.3 - 2020-08-20

- `Eu4Date::eu4_fmt` omits leading zeros in date
- Building information extraction from saves

## v0.1.2 - 2020-08-15

- Bump jomini to v0.2

## v0.1.1 - 2020-08-11

- Rerun the build when ironman token env var changes
- Encapsulate internals of Eu4Date
- Add length check when deserializing country tags

## v0.1.0 - 2020-08-10

Init commit
