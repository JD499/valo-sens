import pandas as pd
import requests
from bs4 import BeautifulSoup

import scraper


def main():
    url = "https://prosettings.net/lists/valorant/"
    headers = {
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) "
        "Chrome/121.0.0.0 Safari/537.36"
    }
    page = requests.get(url, headers=headers)

    if page.status_code == 200:
        soup = BeautifulSoup(page.content, "html.parser")
        player_data = scraper.scrape_data(soup)

        df = pd.DataFrame(player_data)
        df["edpi"] = pd.to_numeric(df["edpi"], errors="coerce")

        grouped = df.groupby("team")

        # Finding the player with minimum eDPI in each team
        min_edpi = (
            grouped.apply(
                lambda x: x.nsmallest(1, "edpi")[["name", "edpi"]], include_groups=False
            )
            .reset_index()
            .drop(columns="level_1")
        )

        min_edpi_sorted = min_edpi.sort_values(by="edpi", ascending=False)

        # Finding the player with maximum eDPI in each team
        max_edpi = (
            grouped.apply(
                lambda x: x.nlargest(1, "edpi")[["name", "edpi"]], include_groups=False
            )
            .reset_index()
            .drop(columns="level_1")
        )

        max_edpi_sorted = max_edpi.sort_values(by="edpi", ascending=False)

        # Finding the mean and median eDPI in each team
        mean_edpi_by_team = grouped["edpi"].mean().sort_values(ascending=False)
        median_edpi_by_team = grouped["edpi"].median().sort_values(ascending=False)

        # Finding the player with the highest and lowest eDPI overall
        highest_edpi_player = df.loc[df["edpi"].idxmax()]
        lowest_edpi_player = df.loc[df["edpi"].idxmin()]

        # Finding the mean and median eDPI overall
        mean_edpi = df["edpi"].mean()
        median_edpi = df["edpi"].median()

        # Printing the results
        print("\nMinimum eDPI by Team:\n")
        print(min_edpi_sorted)
        print("\n-------------------------\n")
        print("Maximum eDPI by Team:\n")
        print(max_edpi_sorted)
        print("\n-------------------------\n")
        print("Mean eDPI by Team:\n")
        print(mean_edpi_by_team)
        print("\n-------------------------\n")
        print("Median eDPI by Team:\n")
        print(median_edpi_by_team)
        print("\n-------------------------\n")
        print("Overall Highest eDPI Player:\n")
        print(highest_edpi_player)
        print("\n-------------------------\n")
        print("Overall Lowest eDPI Player:\n")
        print(lowest_edpi_player)
        print("\n-------------------------\n")
        print("Overall Mean eDPI:\n")
        print(mean_edpi)
        print("\n-------------------------\n")
        print("Overall Median eDPI:\n")
        print(median_edpi)
        print("\n-------------------------\n")


if __name__ == "__main__":
    main()
