# Information Retrieval

## About
This is a collection of solutions to university
assignments from the course on Information Retrieval (IR).
Tasks range from simple word dictionary to querying using binary operators, implementing index compression and TF-IDF.

Solutions were tested on Shakespeare's works, free books from
[Project Gutenberg](https://www.gutenberg.org/) and some
additional datasets. 
They are not polished or guaranteed to be correct.

## Description

### PW1
Creates a simple dictionary with each word occurrence count.

To improve performance for large datasets it's multithreaded and uses memory mapping for reading files. All further solutions use this as a foundation.

### PW2
Uses inverted index and incidence matrix to implement queries with boolean operators such as `&`, `|`, `!`, `\` (and, or, not, and subtraction respectively) and brackets `(`, `)`.
The duration that each query took is measured, which stays the same for further solutions.

**Example:**
```
heaven & hell
father & (brother | sister)
heaven & !hell 
```

### PW3
Uses an inverted positional index. In addition to the previous implementation, it allows searching for words within a specific radius and literal phrases.
There's also a simpler implementation of phrase search using word pair index.

**Example:**
```
# words within radius 3 (inclusive)
hello {3} world
# adjacent words (both directions)
hello {1} world

# words in a specific order
hello > world
# operator can be chained
what > is > love

# phrase literal that desugars to the same thing
"what is love"
```

### PW5
Brings improvements to indexing speed and merges indices from separate files in parallel. 
Allows to index large amounts of data (more than available RAM).
It also measures indexing time, index size, and amount of data.

### PW6
Builds on the previous work and implements index compression. File contains word dictionary packed in a fashion similar to a radix tree; null byte separator; then term positions in variable byte encoding.

### PW7
Implements IR in structured documents by splitting the file into segments like filename, title, authors, body, etc. And by assigning different weights to each part. Plain text and .fb2 files are supported.

### PW8
Implements search in vector space using cosine similarity and TF-IDF. Produces similar documents as query result using clusterization.
