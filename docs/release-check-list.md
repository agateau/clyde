## Prepare

- [ ] Setup

    pip install invoke
    export VERSION=<the-new-version>

- [ ] Prepare

    invoke prepare-release

## Tag

- [ ] Wait for CI to be happy

- [ ] Create tag

    invoke tag

## Publish

- [ ] Download artifacts

    invoke download-artifacts

- [ ] Publish

    invoke publish

## Post publish

- [ ] Update package on clyde-store

    invoke update-store

- [ ] Report tasks and checklist changes

- [ ] Bump version to x.y.z+1-alpha.1

- [ ] Write blog post

