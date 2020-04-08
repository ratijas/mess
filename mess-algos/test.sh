#!/usr/bin/env bash

curl -s http://localhost:3000/login --data '{"username": "ratijas"}'
echo
curl -s http://localhost:3000/login --data '{"username": "jessica"}'
echo
curl -s http://localhost:3000/sendFile
echo
