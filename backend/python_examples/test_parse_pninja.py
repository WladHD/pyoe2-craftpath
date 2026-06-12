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
    # poe.ninja prunes items that have not traded recently, so only the three
    # reference currencies are guaranteed to stay listed - probe those plus
    # the overall parse result instead of asserting on tradeable item names
    print(f"Parsed {len(economy.cache_market_prices)} market prices")
    assert len(economy.cache_market_prices) > 0

    test_divine = economy.cache_market_prices.get(pc.ItemName("Divine Orb"))
    test_exalted = economy.cache_market_prices.get(pc.ItemName("Exalted Orb"))
    test_chaos = economy.cache_market_prices.get(pc.ItemName("Chaos Orb"))

    assert (test_divine != None)
    assert (test_exalted != None)
    assert (test_chaos != None)

    # pick a tradeable item from the parsed data for the conversion checks
    example_name, example_price = next(iter(economy.cache_market_prices.items()))
    print(f"Converting prices of '{example_name.raw_value}'")

    print(economy.currency_convert(example_price, pc.PriceKind.Divine))
    print(economy.currency_convert(example_price, pc.PriceKind.Exalted))
    print(economy.currency_convert(example_price, pc.PriceKind.Chaos))

    # wont write assert for that since its float comp
    print(example_price == example_price)
    print(example_price.get_divine_value() == example_price.get_divine_value())

    assert (pc.PriceInDivines(5) > pc.PriceInDivines(4))
    assert (pc.PriceInDivines(3) < pc.PriceInDivines(4))
    assert (pc.PriceInDivines(4) < pc.PriceInDivines(5))


if __name__ == "__main__":
    main()
