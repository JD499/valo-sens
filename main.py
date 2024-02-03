import requests
from bs4 import BeautifulSoup
import scraper


def main():
    url = "https://prosettings.net/lists/valorant/"
    headers = {
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3'}
    page = requests.get(url, headers=headers)

    if page.status_code == 200:
        soup = BeautifulSoup(page.content, 'html.parser')
        player_data = scraper.scrape_data(soup)
        print(player_data)


if __name__ == "__main__":
    main()
