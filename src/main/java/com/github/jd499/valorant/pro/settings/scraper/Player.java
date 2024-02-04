package com.github.jd499.valorant.pro.settings.scraper;

/**
 * The player class.
 *
 * @param team
 * @param name
 * @param dpi
 * @param sensitivity
 * @param edpi
 */
public record Player(String team, String name, String dpi, String sensitivity, String edpi) {

  /**
   * @return a string representation of the object
   */
  @Override
  public String toString() {
    return "Player{"
        + "team='"
        + team
        + '\''
        + ", name='"
        + name
        + '\''
        + ", dpi='"
        + dpi
        + '\''
        + ", sensitivity='"
        + sensitivity
        + '\''
        + ", edpi='"
        + edpi
        + '\''
        + '}';
  }
}
