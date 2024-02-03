def scrape_data(soup):
    players = []
    try:
        table = soup.find("table", id="pro-list-table")
        rows = table.find_all("tr")[1:]  # Skip the header row

        for row in rows:
            columns = row.find_all("td")
            if len(columns) >= 8:
                # prosettings.net may list some individuals as part of a team, even
                # though they do not participate as professional players. Account for this when extracting
                # data, but note that any player filtering applied here may not always be accurate for this reason.
                #
                # To implement such a filter, one could check if a player is a "free agent", "retired", or a
                # "content creator". If so, skip their data in the scraping process. Here, the relevant code
                # for this functionality is commented out:
                #
                # status = columns[1].text.strip().lower()
                # if "free agent" in status or "retired" in status or "content creator" in status:
                #     continue  # Skip this player

                player_info = {
                    "team": columns[1].text.strip(),
                    "name": columns[2].text.strip(),
                    "dpi": columns[5].text.strip(),
                    "sensitivity": columns[6].text.strip(),
                    "edpi": columns[7].text.strip(),
                }
                if all(player_info.values()):
                    players.append(player_info)
    except Exception as e:
        print(f"An error occurred: {e}")
    return players
