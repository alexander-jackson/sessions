# Blackboards

Blackboards is a basic website for booking training times for members of Warwick
Barbell. It was used initially for taster sessions, where the club would run
short introduction to lifting events and people who were interested could book
a space.

Due to social distancing measures, we needed to be able to cap the number of
signups, and ensure that people were signing up with valid emails. We also only
wanted prospective members to come to a single session, so that we would have
as much space as possible.

## Dependencies

Blackboards runs on the nightly version of Rust and requires `sqlite3` to be
installed currently.

Rust can be installed from `https://www.rust-lang.org/learn/get-started`, at
which point you can run `rustup default nightly` to get the latest nightly
compiler.

## Usage

Setting up the website locally is designed to be quite simple, first clone the
repository and enter the directory:

```bash
git clone git@github.com:alexander-jackson/blackboards.git
cd blackboards
```

Then set up the database and run the project:

```bash
make database
cargo run
```

You should then be able to go to `http://localhost:8000/sessions` to see the
website.

## Sending Emails

Sending of emails is by default turned off, and none will be sent. This is
controlled by a `.env` file with the following format:

```bash
SEND_EMAILS=<anything>
FROM_ADDRESS=<your_email_address>
FROM_NAME=<your_name>
APP_PASSWORD=<google_app_password>
```

`SEND_EMAILS` only needs to be specified, it doesn't matter what value it has.
`FROM_ADDRESS` should currently be a `gmail.com` address. `FROM_NAME` can be
anything, but it's what people will see when they receive the email.
`APP_PASSWORD` can be generated by Google as per
[here](https://support.google.com/accounts/answer/185833?hl=en).
