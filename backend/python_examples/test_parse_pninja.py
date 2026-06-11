import pyoe2_craftpath as pc

# You probably want to compare file timestamp to check if old cache is older than ... idk .. 1h?
# Example of that below
# For info of what is available visit https://poe.ninja/poe2/economy/
MARKET_MAP = {
    "./cache/pn_abyss.json": "https://poe.ninja/poe2/api/economy/exchange/current/overview?league=Standard&type=Abyss",
    "./cache/pn_currency.json": "https://poe.ninja/poe2/api/economy/exchange/current/overview?league=Standard&type=Currency",
    "./cache/pn_essences.json": "https://poe.ninja/poe2/api/economy/exchange/current/overview?league=Standard&type=Essences",
    "./cache/pn_ritual.json": "https://poe.ninja/poe2/api/economy/exchange/current/overview?league=Standard&type=Ritual"
}

CACHE_TTL_IN_SECONDS = 60 * 60  # 1 hour in seconds


def main():
    raw_fetched_responses = pc.retrieve_contents_from_urls_with_cache_unstable_order(
        cache_url_map=MARKET_MAP,
        max_cache_duration_in_sec=CACHE_TTL_IN_SECONDS
    )

    ########################################
    ###     this is the magic line       ###
    ########################################
    economy = pc.PoeNinjaMarketPriceProvider.parse_from_json_list(
        raw_fetched_responses)

    # everything else just checks validity
    test_currency = economy.cache_market_prices.get(
        pc.ItemName("Perfect Orb of Transmutation"))
    test_ritual = economy.cache_market_prices.get(
        pc.ItemName("Omen of the Blackblooded"))
    test_essence = economy.cache_market_prices.get(
        pc.ItemName("Perfect Essence of Ruin"))
    test_abyss = economy.cache_market_prices.get(
        pc.ItemName("Kulemak's Invitation"))

    assert (test_currency != None)
    assert (test_ritual != None)
    assert (test_essence != None)
    assert (test_abyss != None)

    print(economy.currency_convert(test_abyss, pc.PriceKind.Divine))  # no change
    print(economy.currency_convert(test_abyss, pc.PriceKind.Exalted))
    print(economy.currency_convert(test_abyss, pc.PriceKind.Chaos))

    # wont write assert for that since its float comp
    print(test_abyss == test_abyss)
    print(test_abyss.get_divine_value() == test_abyss.get_divine_value())

    assert (pc.PriceInDivines(5) > pc.PriceInDivines(4))
    assert (pc.PriceInDivines(3) < pc.PriceInDivines(4))
    assert (pc.PriceInDivines(4) < pc.PriceInDivines(5))


if __name__ == "__main__":
    main()
