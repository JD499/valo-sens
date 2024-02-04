# Valorant Pro Settings Scraper

## Introduction
Valorant Pro Settings Scraper is a Java-based application designed to scrape professional Valorant players' settings from [prosettings.net](https://prosettings.net/lists/valorant/). It processes the data to calculate and display various statistics, including the minimum and maximum eDPI values per team, mean and median eDPI values, and the overall highest and lowest eDPI players among professional Valorant players.

## Features
- Web scraping using Jsoup
- Data processing with Java Streams
- Statistical calculations using Apache Commons Math

## Installation

### Java Version
Requires Java to be installed on your system.
1. Download `ValorantProSettingsScraper-all.jar`.
2. Open a terminal or command prompt.
3. Navigate to the download location.
4. Run the JAR file using the command:
   ```bash
   java -jar valorantProSettingsScraper-all.jar
   ```

### Windows Version
1. Download `jpackageImage.zip`.
2. Extract the `valorantProSettingsScraper` directory from the zip file.
3. Navigate to the extracted directory and run `valorantProSettingsScraper.exe`.

## Usage
Run the application as per the above installation instructions. The program will scrape data from the specified source and display it in the console.
