/**
 * All code in this file originates from pacman's lib/libalpm/version.c
 */


/**
 * Compare alpha and numeric segments of two versions.
 * return 1: a is newer than b
 *        0: a and b are the same version
 *       -1: b is newer than a
 */
int rpmvercmp(const char *a, const char *b)
{
    char oldch1, oldch2;
    char *str1, *str2;
    char *ptr1, *ptr2;
    char *one, *two;
    int rc;
    int isnum;
    int ret = 0;

    /* easy comparison to see if versions are identical */
    if(strcmp(a, b) == 0) return 0;

    str1 = strdup(a);
    str2 = strdup(b);

    one = ptr1 = str1;
    two = ptr2 = str2;

    /* loop through each version segment of str1 and str2 and compare them */
    while (*one && *two) {
        while (*one && !isalnum((int)*one)) one++;
        while (*two && !isalnum((int)*two)) two++;

        /* If we ran to the end of either, we are finished with the loop */
        if (!(*one && *two)) break;

        /* If the separator lengths were different, we are also finished */
        if ((one - ptr1) != (two - ptr2)) {
            ret = (one - ptr1) < (two - ptr2) ? -1 : 1;
            goto cleanup;
        }

        ptr1 = one;
        ptr2 = two;

        /* grab first completely alpha or completely numeric segment */
        /* leave one and two pointing to the start of the alpha or numeric */
        /* segment and walk ptr1 and ptr2 to end of segment */
        if (isdigit((int)*ptr1)) {
            while (*ptr1 && isdigit((int)*ptr1)) ptr1++;
            while (*ptr2 && isdigit((int)*ptr2)) ptr2++;
            isnum = 1;
        } else {
            while (*ptr1 && isalpha((int)*ptr1)) ptr1++;
            while (*ptr2 && isalpha((int)*ptr2)) ptr2++;
            isnum = 0;
        }

        /* save character at the end of the alpha or numeric segment */
        /* so that they can be restored after the comparison */
        oldch1 = *ptr1;
        *ptr1 = '\0';
        oldch2 = *ptr2;
        *ptr2 = '\0';

        /* this cannot happen, as we previously tested to make sure that */
        /* the first string has a non-null segment */
        if (one == ptr1) {
            ret = -1;	/* arbitrary */
            goto cleanup;
        }

        /* take care of the case where the two version segments are */
        /* different types: one numeric, the other alpha (i.e. empty) */
        /* numeric segments are always newer than alpha segments */
        /* XXX See patch #60884 (and details) from bugzilla #50977. */
        if (two == ptr2) {
            ret = isnum ? 1 : -1;
            goto cleanup;
        }

        if (isnum) {
            /* this used to be done by converting the digit segments */
            /* to ints using atoi() - it's changed because long  */
            /* digit segments can overflow an int - this should fix that. */

            /* throw away any leading zeros - it's a number, right? */
            while (*one == '0') one++;
            while (*two == '0') two++;

            /* whichever number has more digits wins */
            if (strlen(one) > strlen(two)) {
                ret = 1;
                goto cleanup;
            }
            if (strlen(two) > strlen(one)) {
                ret = -1;
                goto cleanup;
            }
        }

        /* strcmp will return which one is greater - even if the two */
        /* segments are alpha or if they are numeric.  don't return  */
        /* if they are equal because there might be more segments to */
        /* compare */
        rc = strcmp(one, two);
        if (rc) {
            ret = rc < 1 ? -1 : 1;
            goto cleanup;
        }

        /* restore character that was replaced by null above */
        *ptr1 = oldch1;
        one = ptr1;
        *ptr2 = oldch2;
        two = ptr2;
    }

    /* this catches the case where all numeric and alpha segments have */
    /* compared identically but the segment separating characters were */
    /* different */
    if ((!*one) && (!*two)) {
        ret = 0;
        goto cleanup;
    }

    /* the final showdown. we never want a remaining alpha string to
     * beat an empty string. the logic is a bit weird, but:
     * - if one is empty and two is not an alpha, two is newer.
     * - if one is an alpha, two is newer.
     * - otherwise one is newer.
     * */
    if ( (!*one && !isalpha((int)*two))
         || isalpha((int)*one) ) {
        ret = -1;
    } else {
        ret = 1;
    }

    cleanup:
    free(str1);
    free(str2);
    return ret;
}
