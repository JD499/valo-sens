package com.github.jd499.valorant.pro.settings.scraper;

import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import org.jsoup.Jsoup;
import org.jsoup.nodes.Document;
import org.jsoup.nodes.Element;
import org.jsoup.select.Elements;

public class Scraper {

  /**
   * Scrapes the data from the given URL.
   *
   * @param url the URL to scrape
   * @return the list of players
   */
  public static List<Player> scrapeData(String url) {
    List<Player> players = new ArrayList<>();
    try {
      Document doc = Jsoup.connect(url).get();
      Element table = doc.select("table#pro-list-table").first();
      if (table != null) {
        Elements rows = table.select("tr");

        for (Element row : rows) {
          Elements columns = row.select("td");
          if (columns.size() >= 8) {
            String team = columns.get(1).text().trim();
            String name = columns.get(2).text().trim();
            String dpi = columns.get(5).text().trim();
            String sensitivity = columns.get(6).text().trim();
            String edpi = columns.get(7).text().trim();

            // Check if any attribute is missing or empty
            if (!team.isEmpty()
                && !name.isEmpty()
                && !dpi.isEmpty()
                && !sensitivity.isEmpty()
                && !edpi.isEmpty()) {
              Player player = new Player(team, name, dpi, sensitivity, edpi);
              players.add(player);
            }
          }
        }
      } else {
        System.out.println("Table not found on the page");
      }
    } catch (IOException e) {
      e.printStackTrace();
    }
    return players;
  }
}
