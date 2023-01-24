# User & Auth

## `POST` /api/v1/users/userInfo
- Turn a user UUID and token into a user object

## `POST` /api/v1/users/startVerification
- Start the verification process for a user by phone number

## `POST` /api/v1/users/checkVerification
- Check the verification code for a user by phone number

# Posts

## `GET` /api/v1/posts/feedPage/{index}
- Get a page of posts for the feed
- Returns page and next page index

## `GET` /api/v1/posts/single/{uuid}
- Get a single post by UUID (for viewing)

## `POST` /api/v1/posts/new
- Create a new post

## `POST` /api/v1/posts/claim
- Claim a post

# Other