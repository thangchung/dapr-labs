# Rust CoffeeShop App

## REST APIs

### GET all item-types

<details>
  <summary><b>GET {{host}}/product/v1/api/item-types HTTP/1.1</b></summary>
Output:

```json
[
  {
    "type": 0,
    "name": "CAPPUCCINO"
  },
  {
    "type": 1,
    "name": "COFFEE_BLACK"
  },
  {
    "type": 2,
    "name": "COFFEE_WITH_ROOM"
  },
  {
    "type": 3,
    "name": "ESPRESSO"
  },
  {
    "type": 4,
    "name": "ESPRESSO_DOUBLE"
  },
  {
    "type": 5,
    "name": "LATTE"
  },
  {
    "type": 6,
    "name": "CAKEPOP"
  },
  {
    "type": 7,
    "name": "CROISSANT"
  },
  {
    "type": 8,
    "name": "MUFFIN"
  },
  {
    "type": 9,
    "name": "CROISSANT_CHOCOLATE"
  }
]
```

</details>

### GET items-by-types

<details>
  <summary><b>GET {{host}}/product/v1/api/items-by-types/1,2,3 HTTP/1.1</b></summary>
Output:

```json
[
  {
    "price": 3,
    "type": 1
  },
  {
    "price": 3,
    "type": 2
  },
  {
    "price": 3.5,
    "type": 3
  }
]
```

</details>

### GET all fulfillment-orders

<details>
  <summary><b>GET {{host}}/counter/v1/api/fulfillment-orders HTTP/1.1</b></summary>

Output:

```json
[
    {
    "orderSource": 0,
    "loyaltyMemberId": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
    "orderStatus": 2,
    "location": 0,
    "lineItems": [
      {
        "itemType": 1,
        "name": "COFFEE_BLACK",
        "price": 0,
        "itemStatus": 2,
        "isBaristaOrder": true,
        "id": "216080bb-4c4c-4d4c-b5c8-c445db1ceff7",
        "created": "2023-05-01T13:20:15.713784Z",
        "updated": null
      },
      {
        "itemType": 4,
        "name": "ESPRESSO_DOUBLE",
        "price": 0,
        "itemStatus": 2,
        "isBaristaOrder": true,
        "id": "8fd64e68-443a-4c3b-86c9-ba7b0de1c43a",
        "created": "2023-05-01T13:20:15.713775Z",
        "updated": null
      },
      {
        "itemType": 7,
        "name": "CROISSANT",
        "price": 0,
        "itemStatus": 2,
        "isBaristaOrder": false,
        "id": "a58d0d33-398e-42ed-ac02-93f1a7a7db71",
        "created": "2023-05-01T13:20:15.716271Z",
        "updated": null
      }
    ],
    "id": "3e678f8b-d78a-42b5-8384-cb0a3684cc01",
    "created": "2023-05-01T13:20:15.709858Z",
    "updated": null
  }
]
```

</details>

### Place an order

<details>
  <summary><b>POST {{host}}/counter/v1/api/orders HTTP/1.1</b></summary>

Input:

```json
{
    "commandType": 0,
    "orderSource": 0,
    "location": 0,
    "loyaltyMemberId": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
    "baristaItems": [
        {
            "itemType": {{$randomInt 0 5}}
        }
    ],
    "kitchenItems": [
        {
        "itemType": {{$randomInt 6 9}}
        }
    ],
    "timestamp": "2022-07-04T11:38:00.210Z"
}
```

Output:

```json
```

</details>
