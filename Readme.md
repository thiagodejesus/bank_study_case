# A simple bank system

Requirements

- Account registration

- Transactions

- Concurrency of transactions

## How to Run

Run the below command that will start a postgres container

```sh
	docker run -d \
	--name pg \
	-e POSTGRES_PASSWORD=pass \
	-e POSTGRES_DB=db \
	-e POSTGRES_USER=user \
	-p 5432:5432 \
	postgres
```

to test the above pg container, run the following command

````
psql -h localhost -U user -d db

```sh
