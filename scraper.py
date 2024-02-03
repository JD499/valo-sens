def scrape_data(soup):
    players = []
    try:
        table = soup.find('table', id='pro-list-table')
        rows = table.find_all('tr')[1:]  # Skip the header row

        for row in rows:
            columns = row.find_all('td')
            if len(columns) >= 8:
                # Check for free agent, retired, or content creator status
                status = columns[1].text.strip().lower()
                if "free agent" in status or "retired" in status or "content creator" in status:
                    continue  # Skip this player

                player_info = {
                    'name': columns[2].text.strip(),
                    'dpi': columns[5].text.strip(),
                    'sensitivity': columns[6].text.strip(),
                    'edpi': columns[7].text.strip()
                }
                if all(player_info.values()):
                    players.append(player_info)
    except Exception as e:
        print(f"An error occurred: {e}")
    return players
