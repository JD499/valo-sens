package com.github.jd499.valorant.pro.settings.scraper;

import java.util.*;
import java.util.Scanner;
import java.util.stream.Collectors;
import org.apache.commons.math3.stat.descriptive.DescriptiveStatistics;
import org.apache.commons.math3.stat.descriptive.rank.Median;

/** The main class. */
public class Main {
  /**
   * The main method.
   *
   * @param args the input arguments
   */
  public static void main(String[] args) {
    String url = "https://prosettings.net/lists/valorant/";
    List<Player> players = Scraper.scrapeData(url);

    // Finding the player with the minimum eDPI in each team
    Map<String, Player> minEdpiByTeam = findMinOrMaxEdpiByTeam(players, true);

    // Finding the player with the maximum eDPI in each team
    Map<String, Player> maxEdpiByTeam = findMinOrMaxEdpiByTeam(players, false);

    // Calculating mean eDPI by team
    Map<String, Double> meanEdpiByTeam =
        players.stream()
            .collect(
                Collectors.groupingBy(
                    Player::team,
                    Collectors.averagingDouble(player -> Double.parseDouble(player.edpi()))));

    // Calculating median eDPI by team
    Map<String, Double> medianEdpiByTeam =
        players.stream()
            .collect(
                Collectors.groupingBy(
                    Player::team,
                    Collectors.collectingAndThen(
                        Collectors.toList(),
                        list -> {
                          double[] edpis =
                              list.stream()
                                  .mapToDouble(player -> Double.parseDouble(player.edpi()))
                                  .toArray();
                          return new Median().evaluate(edpis);
                        })));

    // Finding the overall highest and lowest eDPI players
    Player highestEdpiPlayer =
        players.stream()
            .max(Comparator.comparing(player -> Double.parseDouble(player.edpi())))
            .orElse(null);
    Player lowestEdpiPlayer =
        players.stream()
            .min(Comparator.comparing(player -> Double.parseDouble(player.edpi())))
            .orElse(null);

    // Calculating overall mean and median eDPI
    DescriptiveStatistics stats = new DescriptiveStatistics();
    players.forEach(player -> stats.addValue(Double.parseDouble(player.edpi())));
    double overallMeanEdpi = stats.getMean();
    double overallMedianEdpi = stats.getPercentile(50);

    // Printing results
    printResultsSortedByEdpi("Minimum eDPI by Team", minEdpiByTeam, true);
    printResultsSortedByEdpi("Maximum eDPI by Team", maxEdpiByTeam, false);
    printTeamStatistics("Mean eDPI by Team", meanEdpiByTeam);
    printTeamStatistics("Median eDPI by Team", medianEdpiByTeam);
    printPlayer("Overall Highest eDPI Player", highestEdpiPlayer);
    printPlayer("Overall Lowest eDPI Player", lowestEdpiPlayer);
    System.out.println("Overall Mean eDPI: " + overallMeanEdpi);
    System.out.println("Overall Median eDPI: " + overallMedianEdpi);

    // Wait for user to press Enter before exiting
    System.out.println("Press Enter to exit...");
    try (Scanner scanner = new Scanner(System.in)) {
      scanner.nextLine();
    }
  }

  /**
   * Finds the player with the minimum or maximum eDPI in each team.
   *
   * @param players the list of players
   * @param findMin if true, finds the player with the minimum eDPI; otherwise, finds the player
   *     with the maximum eDPI
   * @return a map with the team name as the key and the player with the minimum or maximum eDPI as
   *     the value
   */
  private static Map<String, Player> findMinOrMaxEdpiByTeam(List<Player> players, boolean findMin) {
    Map<String, Player> result = new HashMap<>();
    for (Player player : players) {
      result.compute(
          player.team(),
          (team, currentBest) -> {
            if (currentBest == null) {
              return player;
            }
            double currentPlayerEdpi = Double.parseDouble(player.edpi());
            double currentBestEdpi = Double.parseDouble(currentBest.edpi());
            if ((findMin && currentPlayerEdpi < currentBestEdpi)
                || (!findMin && currentPlayerEdpi > currentBestEdpi)) {
              return player;
            }
            return currentBest;
          });
    }
    return result;
  }

  /**
   * Prints the results sorted by eDPI.
   *
   * @param title the title of the results
   * @param results the results to print
   * @param ascending if true, sorts the results in ascending order; otherwise, sorts the results in
   *     descending order
   */
  private static void printResultsSortedByEdpi(
      String title, Map<String, Player> results, boolean ascending) {
    System.out.println("\n" + title + ":\n");
    Comparator<Map.Entry<String, Player>> comparator =
        Map.Entry.comparingByValue(
            Comparator.comparing(player -> Double.parseDouble(player.edpi())));
    if (!ascending) {
      comparator = comparator.reversed();
    }
    results.entrySet().stream()
        .sorted(comparator)
        .forEach(entry -> System.out.println("Team: " + entry.getKey() + " - " + entry.getValue()));
  }

  /**
   * Prints the team statistics.
   *
   * @param title the title of the statistics
   * @param statistics the statistics to print
   */
  private static void printTeamStatistics(String title, Map<String, Double> statistics) {
    System.out.println("\n" + title + ":\n");
    statistics.entrySet().stream()
        .sorted(Map.Entry.<String, Double>comparingByValue().reversed())
        .forEach(entry -> System.out.println("Team: " + entry.getKey() + " - " + entry.getValue()));
  }

  /**
   * Prints the player.
   *
   * @param title the title of the player
   * @param player the player to print
   */
  private static void printPlayer(String title, Player player) {
    System.out.println("\n" + title + ":\n");
    if (player != null) {
      System.out.println(player);
    } else {
      System.out.println("No data available");
    }
  }
}
