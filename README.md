Rust URL Shortener + Analytics

Production-ready URL shortener and analytics platform built with Rust, Axum, PostgreSQL, SQLx, Docker, and Leptos.
__

Live Demo

Frontend:
https://www.pablomendoza.site
Backend:
https://me.pablomendoza.site/health

__

Why I Built This

I wanted to build a production-grade Rust backend that demonstrates:

HTTP request handling
Subdomain routing
PostgreSQL persistence
Analytics collection
Production debugging
Full-stack integration with Leptos
The project is fully deployed and serves real traffic.
__

Features

Subdomain-based URL redirects
PostgreSQL persistence
Click analytics: Unique visitor tracking, Device analytics, Referrer analytics, SQLx automatic migrations, Dockerized deployment, HTTPS and custom domain support, Production dashboard built with Leptos
__

Tech Stack

Backend: Rust, Axum, Tokio, SQLx, PostgreSQL
Frontend: Leptos, WASM
Infrastructure: Docker, Northflank, Hostinger, GitHub
__

Project Architecture
__

API Endpoints

Method	/Endpoint
GET	/health
GET	/links
GET	/links/count
GET	/analytics/clicks
GET	/analytics/clicks/total
GET	/analytics/visitors/unique
GET	/analytics/devices
GET	/analytics/referrers
__

Database Schema

links:

id
subdomain
destination_url

click_events:

id
subdomain
clicked_at
ip_address
referrer
device_type
__

Production Challenges Solved

Automatic SQLx migrations
TLS-enabled PostgreSQL connections
__


