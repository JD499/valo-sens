#include <ctype.h>
#include <curl/curl.h>
#include <libxml/HTMLparser.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define MAX_PLAYERS 1000
#define MAX_STRING 256

typedef struct {
    char team[MAX_STRING];
    char name[MAX_STRING];
    char dpi[MAX_STRING];
    double edpi;
} Player;

typedef struct {
    char *memory;
    size_t size;
} MemoryStruct;

Player players[MAX_PLAYERS];
int player_count = 0;

static size_t writeMemmoryCallback(const void *contents, size_t size, size_t nmemb, void *userp) {
    size_t realsize = size * nmemb;
    MemoryStruct *mem = userp;

    char *ptr = realloc(mem->memory, mem->size + realsize + 1);
    if (!ptr)
        return 0;

    mem->memory = ptr;
    memcpy(&(mem->memory[mem->size]), contents, realsize);
    mem->size += realsize;
    mem->memory[mem->size] = 0;

    return realsize;
}

static void trim(char *str) {
    if (str == NULL) {
        return;
    }

    char *start = str;
    while (isspace(*start))
        start++;

    if (*start == 0) {
        *str = 0;
        return;
    }

    char *end = start + strlen(start) - 1;
    while (end > start && isspace(*end))
        end--;

    *(end + 1) = 0;

    if (start != str) {
        memmove(str, start, strlen(start) + 1);
    }
}

static void parseTable(const xmlNode *table) {
    for (xmlNode *child = table->children; child; child = child->next) {
        if (child->type == XML_ELEMENT_NODE && xmlStrcmp(child->name, "tbody") == 0) {
            for (xmlNode *row = child->children; row; row = row->next) {
                if (row->type == XML_ELEMENT_NODE && xmlStrcmp(row->name, "tr") == 0) {
                    Player player = {0};
                    int columnIndex = 0;

                    for (xmlNode *column = row->children; column; column = column->next) {
                        if (column->type == XML_ELEMENT_NODE && xmlStrcmp(column->name, "td") == 0) {
                            char *content = (char *) xmlNodeGetContent(column);
                            trim(content);

                            switch (columnIndex) {
                                case 1:
                                    strncpy(player.team, content, MAX_STRING - 1);
                                    break;
                                case 2:
                                    strncpy(player.name, content, MAX_STRING - 1);
                                    break;
                                case 5:
                                    strncpy(player.dpi, content, MAX_STRING - 1);
                                    break;
                                case 7:
                                    player.edpi = atof(content);
                                    break;
                            }
                            xmlFree(content);
                            columnIndex++;
                        }
                    }

                    if (*player.team && *player.name && *player.dpi && player.edpi != 0) {
                        if (player_count < MAX_PLAYERS) {
                            players[player_count++] = player;
                        } else {
                            fprintf(stderr, "Maximum player limit reached!\n");
                            return;
                        }
                    }
                }
            }
        }
    }
}

static void traverseNodes(const xmlNode *node) {
    for (; node; node = node->next) {
        if (node->type == XML_ELEMENT_NODE && xmlStrcmp(node->name, "table") == 0) {
            parseTable(node);
        }
        traverseNodes(node->children);
    }
}

static void parsePlayers(const char *html) {
    htmlDocPtr doc = htmlReadMemory(html, strlen(html), NULL, NULL,
                                    HTML_PARSE_NOERROR | HTML_PARSE_NOWARNING | HTML_PARSE_RECOVER);
    if (doc) {
        xmlNode *root = xmlDocGetRootElement(doc);
        traverseNodes(root);
        xmlFreeDoc(doc);
    }
}

static int compareEDPI(const void *a, const void *b) { return ((Player *) a)->edpi - ((Player *) b)->edpi; }

static void findMinMaxEDPIByTeam() {
    printf("\nMinimum and Maximum eDPI by Team:\n");
    char current_team[MAX_STRING] = "";
    int i = 0;

    while (i < player_count) {

        Player min_player = players[i];
        Player max_player = players[i];

        strcpy(current_team, players[i].team);


        while (i < player_count && strcmp(players[i].team, current_team) == 0) {
            if (players[i].edpi < min_player.edpi) {
                min_player = players[i];
            }
            if (players[i].edpi > max_player.edpi) {
                max_player = players[i];
            }
            i++;
        }


        printf("Team: %s\n", current_team);
        printf("Min: %s (eDPI: %.2f)\n", min_player.name, min_player.edpi);
        printf("Max: %s (eDPI: %.2f)\n", max_player.name, max_player.edpi);
    }
}


typedef struct {
    char team[MAX_STRING];
    double mean_edpi;
} TeamStats;

static int compareTeamMean(const void *a, const void *b) {

    return ((TeamStats *) b)->mean_edpi - ((TeamStats *) a)->mean_edpi > 0 ? 1 : -1;
}

static void calculateTeamsMean() {
    TeamStats team_stats[MAX_PLAYERS];
    int team_count = 0;


    char current_team[MAX_STRING] = "";
    double sum_edpi = 0;
    int player_count_in_team = 0;

    for (int i = 0; i < player_count; i++) {
        if (strcmp(players[i].team, current_team) != 0) {

            if (team_count > 0) {
                team_stats[team_count - 1].mean_edpi = sum_edpi / player_count_in_team;
            }


            strcpy(current_team, players[i].team);
            sum_edpi = players[i].edpi;
            player_count_in_team = 1;
            strncpy(team_stats[team_count].team, current_team, MAX_STRING - 1);
            team_count++;
        } else {

            sum_edpi += players[i].edpi;
            player_count_in_team++;
        }
    }


    if (team_count > 0) {
        team_stats[team_count - 1].mean_edpi = sum_edpi / player_count_in_team;
    }


    qsort(team_stats, team_count, sizeof(TeamStats), compareTeamMean);


    printf("\nMean eDPI by Team (Highest to Lowest):\n");
    for (int i = 0; i < team_count; i++) {
        printf("Team: %s - %.2f\n", team_stats[i].team, team_stats[i].mean_edpi);
    }
}


static void findOverallStats() {
    if (player_count == 0) {
        printf("No player data available.\n");
        return;
    }


    qsort(players, player_count, sizeof(Player), compareEDPI);

    double sum_edpi = 0;
    for (int i = 0; i < player_count; i++) {
        sum_edpi += players[i].edpi;
    }

    double mean_edpi = sum_edpi / player_count;

    double median_edpi;
    if (player_count % 2 == 0) {
        median_edpi = (players[player_count / 2 - 1].edpi + players[player_count / 2].edpi) / 2.0;
    } else {
        median_edpi = players[player_count / 2].edpi;
    }

    printf("\nOverall Statistics:\n");
    printf("Lowest eDPI: %s (Team: %s, eDPI: %.2f)\n", players[0].name, players[0].team, players[0].edpi);
    printf("Highest eDPI: %s (Team: %s, eDPI: %.2f)\n", players[player_count - 1].name, players[player_count - 1].team,
           players[player_count - 1].edpi);
    printf("Mean eDPI: %.2f\n", mean_edpi);
    printf("Median eDPI: %.2f\n", median_edpi);
}


static void displayPlayerInfo(const Player *player) {
    printf("\nPlayer Information:\n");
    printf("Name: %s\n", player->name);
    printf("Team: %s\n", player->team);
    printf("eDPI: %.2f\n", player->edpi);
    printf("\nSensitivity at Different DPIs:\n");
    printf("400 DPI: %.3f\n", player->edpi / 400);
    printf("800 DPI: %.3f\n", player->edpi / 800);
    printf("1600 DPI: %.3f\n", player->edpi / 1600);
}

static void displayrandomPlayer() {
    if (player_count == 0) {
        printf("No player data available.\n");
        return;
    }
    int random_index = rand() % player_count;
    displayPlayerInfo(&players[random_index]);
}

static void findClosestPlayer(double target_edpi, int count) {
    double differences[MAX_PLAYERS];
    for (int i = 0; i < player_count; i++) {
        differences[i] = fabs(players[i].edpi - target_edpi);
    }

    int indices[MAX_PLAYERS];
    for (int i = 0; i < player_count; i++)
        indices[i] = i;

    for (int i = 0; i < count && i < player_count; i++) {
        for (int j = i + 1; j < player_count; j++) {
            if (differences[indices[i]] > differences[indices[j]]) {
                int temp = indices[i];
                indices[i] = indices[j];
                indices[j] = temp;
            }
        }
    }

    printf("\nPlayers closest to eDPI %.2f:\n", target_edpi);
    for (int i = 0; i < count && i < player_count; i++) {
        printf("%s (Team: %s, eDPI: %.2f)\n", players[indices[i]].name, players[indices[i]].team,
               players[indices[i]].edpi);
    }
}

static void getPlayerByName(const char *name) {

    for (int i = 0; i < player_count; i++) {

        if (strcasecmp(players[i].name, name) == 0) {
            displayPlayerInfo(&players[i]);
            return;
        }
    }
    printf("Player not found.\n");
}

int main(void) {
    MemoryStruct chunk = {malloc(1), 0};

    curl_global_init(CURL_GLOBAL_DEFAULT);
    CURL *curl = curl_easy_init();
    if (curl) {
        curl_easy_setopt(curl, CURLOPT_URL, "https://prosettings.net/lists/valorant/");
        curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, writeMemmoryCallback);
        curl_easy_setopt(curl, CURLOPT_WRITEDATA, (void *) &chunk);
        curl_easy_setopt(curl, CURLOPT_USERAGENT,
                         "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, "
                         "like Gecko) Chrome/91.0.4472.124 Safari/537.36");

        CURLcode res = curl_easy_perform(curl);
        if (res != CURLE_OK) {
            fprintf(stderr, "curl_easy_perform() failed: %s\n", curl_easy_strerror(res));
        } else {
            parsePlayers(chunk.memory);
            findMinMaxEDPIByTeam();
            calculateTeamsMean();
            findOverallStats();

            char input[MAX_STRING];
            while (1) {
                printf("\nEnter a player's name, a number for eDPI comparison, or "
                       "press Enter for a random player (type 'exit' to quit): ");
                if (fgets(input, sizeof(input), stdin) == NULL)
                    break;
                input[strcspn(input, "\n")] = 0;

                if (strcmp(input, "exit") == 0)
                    break;
                if (strlen(input) == 0) {
                    displayrandomPlayer();
                } else {
                    char *endptr;
                    double edpi = strtod(input, &endptr);
                    if (*endptr == '\0') {
                        findClosestPlayer(edpi, 10);
                    } else {
                        getPlayerByName(input);
                    }
                }
            }
        }

        curl_easy_cleanup(curl);
    }

    free(chunk.memory);
    curl_global_cleanup();
    return EXIT_SUCCESS;
}
